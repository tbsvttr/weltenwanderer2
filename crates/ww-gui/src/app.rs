//! Application state shared across all screens.

use std::path::PathBuf;

use ww_core::World;
use ww_core::entity::EntityId;

use crate::theme::font::PixelFont;
use crate::theme::sprites::SpriteSet;

/// Shared application state accessible by all screens.
pub struct AppState {
    /// The loaded world, if any.
    pub world: Option<World>,
    /// Path to the world directory.
    pub world_dir: Option<PathBuf>,
    /// The pixel font atlas.
    pub font: PixelFont,
    /// The sprite icon set.
    pub sprites: SpriteSet,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Currently selected entity ID (for detail view).
    pub selected_entity: Option<EntityId>,
    /// Entity list search/filter query.
    pub search_query: String,
    /// Cursor index in the entity list.
    pub list_cursor: usize,
}

impl AppState {
    /// Create a new app state with the given font and sprites.
    pub fn new(font: PixelFont, sprites: SpriteSet) -> Self {
        Self {
            world: None,
            world_dir: None,
            font,
            sprites,
            should_quit: false,
            selected_entity: None,
            search_query: String::new(),
            list_cursor: 0,
        }
    }

    /// Load a world from the given directory.
    pub fn load_world(&mut self, dir: &std::path::Path) -> Result<(), String> {
        let result = ww_dsl::compile_dir(dir);
        if result.has_errors() {
            let msgs: Vec<String> = result.diagnostics.iter().map(|d| format!("{d}")).collect();
            return Err(msgs.join("; "));
        }
        self.world = Some(result.world);
        self.world_dir = Some(dir.to_path_buf());
        self.selected_entity = None;
        self.search_query.clear();
        self.list_cursor = 0;
        Ok(())
    }
}
