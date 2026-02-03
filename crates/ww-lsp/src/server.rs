use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use ww_dsl::diagnostics::Severity;

/// Tracks where each file's content sits within the concatenated source.
struct FileSlice {
    uri: Url,
    offset: usize,
    len: usize,
    text: String,
}

/// Info about a compiled entity, now with its defining file URI.
struct EntityInfo {
    name: String,
    kind: String,
    /// Byte span within that file's own text (not the concatenated source).
    local_span: std::ops::Range<usize>,
    /// Which file this entity is defined in.
    uri: Url,
}

/// Shared workspace state.
struct WorkspaceState {
    /// Open document texts (in-memory, may be unsaved).
    open_docs: HashMap<Url, String>,
    /// All entities from the last successful workspace compilation.
    entities: Vec<EntityInfo>,
    /// All entity names (for autocomplete).
    entity_names: Vec<String>,
    /// Workspace root path.
    root: Option<PathBuf>,
}

pub struct WwLanguageServer {
    client: Client,
    state: Arc<RwLock<WorkspaceState>>,
}

impl WwLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: Arc::new(RwLock::new(WorkspaceState {
                open_docs: HashMap::new(),
                entities: Vec::new(),
                entity_names: Vec::new(),
                root: None,
            })),
        }
    }

    /// Recompile the entire workspace and publish diagnostics for all files.
    async fn analyze_workspace(&self) {
        let state = self.state.read().await;
        let root = match &state.root {
            Some(r) => r.clone(),
            None => return,
        };
        let open_docs = state.open_docs.clone();
        drop(state);

        // Discover all .ww files recursively
        let mut file_paths: Vec<PathBuf> = Vec::new();
        collect_ww_files(&root, &mut file_paths);
        file_paths.sort();

        // Build file slices: use open doc text if available, otherwise read from disk
        let mut slices: Vec<FileSlice> = Vec::new();
        let mut concatenated = String::new();

        for path in &file_paths {
            let uri = match Url::from_file_path(path) {
                Ok(u) => u,
                Err(()) => match Url::parse(&format!("file://{}", path.display())) {
                    Ok(u) => u,
                    Err(_) => continue,
                },
            };

            let text = if let Some(open_text) = open_docs.get(&uri) {
                open_text.clone()
            } else {
                match std::fs::read_to_string(path) {
                    Ok(t) => t,
                    Err(_) => continue,
                }
            };

            let len = text.len();

            if !concatenated.is_empty() {
                concatenated.push('\n');
            }
            let offset = concatenated.len();
            concatenated.push_str(&text);

            slices.push(FileSlice {
                uri,
                offset,
                len,
                text,
            });
        }

        if concatenated.is_empty() {
            return;
        }

        // Compile the whole workspace as one source
        let result = ww_dsl::compile_source(&concatenated);

        // Map diagnostics back to per-file
        let mut per_file_diags: HashMap<Url, Vec<Diagnostic>> = HashMap::new();
        // Initialize empty diagnostics for all files (to clear old errors)
        for slice in &slices {
            per_file_diags.entry(slice.uri.clone()).or_default();
        }

        for diag in &result.diagnostics {
            if let Some(slice) = find_slice_for_span(&slices, &diag.span) {
                let local_start = diag.span.start.saturating_sub(slice.offset);
                let local_end = diag.span.end.saturating_sub(slice.offset).min(slice.len);
                let local_span = local_start..local_end;

                let range = byte_span_to_range(&slice.text, &local_span);
                let severity = match diag.severity {
                    Severity::Error => Some(DiagnosticSeverity::ERROR),
                    Severity::Warning => Some(DiagnosticSeverity::WARNING),
                };

                per_file_diags
                    .entry(slice.uri.clone())
                    .or_default()
                    .push(Diagnostic {
                        range,
                        severity,
                        source: Some("ww".into()),
                        message: diag.message.clone(),
                        ..Default::default()
                    });
            }
        }

        // Build entity info
        let mut entities = Vec::new();
        let mut entity_names = Vec::new();

        for entity in result.world.all_entities() {
            entity_names.push(entity.name.clone());

            // Find the entity *definition* (the "Name is a/an kind" line)
            // by searching for "<name> is a" or "<name> is an" pattern,
            // rather than just the first occurrence of the name.
            let def_pos = find_definition_offset(&concatenated, &entity.name);
            if let Some(global_start) = def_pos
                && let Some(slice) = find_slice_for_offset(&slices, global_start)
            {
                let local_start = global_start - slice.offset;
                let local_end = local_start + entity.name.len();
                entities.push(EntityInfo {
                    name: entity.name.clone(),
                    kind: entity.kind.to_string(),
                    local_span: local_start..local_end,
                    uri: slice.uri.clone(),
                });
            }
        }

        // Update state
        {
            let mut state = self.state.write().await;
            state.entities = entities;
            state.entity_names = entity_names;
        }

        // Publish diagnostics for each file
        for (uri, diags) in per_file_diags {
            self.client.publish_diagnostics(uri, diags, None).await;
        }
    }
}

/// Recursively collect all .ww files under a directory.
fn collect_ww_files(dir: &PathBuf, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories and common non-source dirs
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
            {
                collect_ww_files(&path, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "ww") {
            out.push(path);
        }
    }
}

/// Find which file slice a byte span falls into.
fn find_slice_for_span<'a>(
    slices: &'a [FileSlice],
    span: &std::ops::Range<usize>,
) -> Option<&'a FileSlice> {
    find_slice_for_offset(slices, span.start)
}

/// Find the byte offset of an entity's definition (the "Name is a ..." declaration).
/// Falls back to the first occurrence if no definition pattern is found.
fn find_definition_offset(text: &str, name: &str) -> Option<usize> {
    let text_lower = text.to_lowercase();
    let name_lower = name.to_lowercase();

    // Search for "<name> is a " or "<name> is an " which marks the definition
    let mut search_from = 0;
    while let Some(found) = text_lower[search_from..].find(&name_lower) {
        let start = search_from + found;
        let after_name = start + name.len();

        // Check what follows the name
        let rest = &text_lower[after_name..];
        if rest.starts_with(" is a ") || rest.starts_with(" is an ") {
            return Some(start);
        }
        search_from = start + 1;
    }

    // Fallback: first occurrence
    text_lower.find(&name_lower)
}

/// Find which file slice a byte offset falls into.
fn find_slice_for_offset(slices: &[FileSlice], offset: usize) -> Option<&FileSlice> {
    slices
        .iter()
        .find(|s| offset >= s.offset && offset < s.offset + s.len)
}

fn byte_span_to_range(text: &str, span: &std::ops::Range<usize>) -> Range {
    let start = byte_offset_to_position(text, span.start);
    let end = byte_offset_to_position(text, span.end);
    Range { start, end }
}

fn byte_offset_to_position(text: &str, offset: usize) -> Position {
    let offset = offset.min(text.len());
    let prefix = &text[..offset];
    let line = prefix.matches('\n').count() as u32;
    let last_newline = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let character = (offset - last_newline) as u32;
    Position { line, character }
}

fn position_to_byte_offset(text: &str, pos: Position) -> usize {
    let mut offset = 0;
    for (i, line) in text.lines().enumerate() {
        if i == pos.line as usize {
            return offset + (pos.character as usize).min(line.len());
        }
        offset += line.len() + 1;
    }
    text.len()
}

static KEYWORDS: &[&str] = &[
    "world",
    "is",
    "a",
    "an",
    "in",
    "member of",
    "located at",
    "allied with",
    "rival of",
    "owned by",
    "led by",
    "based at",
    "involving",
    "references",
    "caused by",
    "date",
    "year",
    "month",
    "day",
    "north to",
    "south to",
    "east to",
    "west to",
    "up to",
    "down to",
    "location",
    "character",
    "faction",
    "event",
    "item",
    "lore",
    "fortress",
    "city",
    "region",
    "room",
    "species",
    "occupation",
    "status",
    "traits",
    "climate",
    "population",
    "terrain",
    "type",
    "rarity",
    "source",
    "values",
    "alignment",
];

#[tower_lsp::async_trait]
impl LanguageServer for WwLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Extract workspace root
        let root = params
            .workspace_folders
            .as_ref()
            .and_then(|folders| folders.first())
            .and_then(|f| f.uri.to_file_path().ok())
            .or_else(|| params.root_uri.as_ref().and_then(|u| u.to_file_path().ok()));

        if let Some(root) = root {
            let mut state = self.state.write().await;
            state.root = Some(root);
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![" ".into()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Weltenwanderer LSP initialized")
            .await;

        // Initial workspace analysis
        self.analyze_workspace().await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        // Store open doc text, then recompile workspace
        {
            let mut state = self.state.write().await;

            // If no workspace root set yet, derive it from the file's parent directory
            if state.root.is_none()
                && let Ok(path) = uri.to_file_path()
                && let Some(parent) = path.parent()
            {
                state.root = Some(parent.to_path_buf());
            }

            state.open_docs.insert(uri, text);
        }

        self.analyze_workspace().await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            {
                let mut state = self.state.write().await;
                state.open_docs.insert(uri, change.text);
            }
            self.analyze_workspace().await;
        }
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        // Recompile on save (disk state may differ from open docs)
        self.analyze_workspace().await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        {
            let mut state = self.state.write().await;
            state.open_docs.remove(&params.text_document.uri);
        }
        self.analyze_workspace().await;
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let state = self.state.read().await;

        let mut items = Vec::new();

        // Add entity names as completions (from entire workspace)
        for (i, name) in state.entity_names.iter().enumerate() {
            items.push(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::REFERENCE),
                sort_text: Some(format!("0{:04}", i)),
                detail: Some("entity".into()),
                ..Default::default()
            });
        }

        // Add keywords
        for (i, kw) in KEYWORDS.iter().enumerate() {
            items.push(CompletionItem {
                label: (*kw).to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                sort_text: Some(format!("1{:04}", i)),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let state = self.state.read().await;

        // Get the text of the current file
        let text = match state.open_docs.get(&uri) {
            Some(t) => t.clone(),
            None => {
                if let Ok(path) = uri.to_file_path() {
                    std::fs::read_to_string(path).unwrap_or_default()
                } else {
                    return Ok(None);
                }
            }
        };

        let offset = position_to_byte_offset(&text, pos);
        let word_at_cursor = find_word_at_offset(&text, offset);

        if word_at_cursor.is_empty() {
            return Ok(None);
        }

        // Search for entity name match (supports cross-file go-to-def)
        for entity in &state.entities {
            let name_lower = entity.name.to_lowercase();
            let text_lower = text.to_lowercase();

            // Check if cursor is inside a reference to this entity
            let mut search_from = 0;
            while let Some(found) = text_lower[search_from..].find(&name_lower) {
                let start = search_from + found;
                let end = start + entity.name.len();
                if offset >= start && offset <= end {
                    let def_range =
                        byte_span_to_range(&get_file_text(&state, &entity.uri), &entity.local_span);
                    return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                        uri: entity.uri.clone(),
                        range: def_range,
                    })));
                }
                search_from = start + 1;
            }
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let state = self.state.read().await;

        let text = match state.open_docs.get(&uri) {
            Some(t) => t.clone(),
            None => {
                if let Ok(path) = uri.to_file_path() {
                    std::fs::read_to_string(path).unwrap_or_default()
                } else {
                    return Ok(None);
                }
            }
        };

        let offset = position_to_byte_offset(&text, pos);

        for entity in &state.entities {
            let text_lower = text.to_lowercase();
            let name_lower = entity.name.to_lowercase();
            let mut search_from = 0;
            while let Some(found) = text_lower[search_from..].find(&name_lower) {
                let start = search_from + found;
                let end = start + entity.name.len();
                if offset >= start && offset <= end {
                    let defined_in = entity
                        .uri
                        .to_file_path()
                        .ok()
                        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
                        .unwrap_or_default();

                    let hover_text = format!(
                        "**{}** [{}]\n\nDefined in `{}`",
                        entity.name, entity.kind, defined_in
                    );
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_text,
                        }),
                        range: Some(byte_span_to_range(&text, &(start..end))),
                    }));
                }
                search_from = start + 1;
            }
        }

        Ok(None)
    }
}

/// Get a file's text from open docs or disk.
fn get_file_text(state: &WorkspaceState, uri: &Url) -> String {
    if let Some(text) = state.open_docs.get(uri) {
        return text.clone();
    }
    if let Ok(path) = uri.to_file_path()
        && let Ok(text) = std::fs::read_to_string(path)
    {
        return text;
    }
    String::new()
}

fn find_word_at_offset(text: &str, offset: usize) -> &str {
    if offset >= text.len() {
        return "";
    }

    let bytes = text.as_bytes();

    let mut start = offset;
    while start > 0 && is_word_char(bytes[start - 1]) {
        start -= 1;
    }

    let mut end = offset;
    while end < bytes.len() && is_word_char(bytes[end]) {
        end += 1;
    }

    &text[start..end]
}

fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
