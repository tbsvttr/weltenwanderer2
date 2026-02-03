# TODO

## Testing

- [ ] CLI command integration tests (`ww-cli/src/commands/` — 10 commands, 0 tests)
- [ ] TUI app state and navigation tests (`ww-cli/src/tui/app.rs`)
- [ ] LSP server tests (`ww-lsp/src/server.rs`)
- [ ] Malformed input recovery tests for parser
- [ ] Round-trip tests: World → JSON → World
- [ ] Large-world stress tests

## LSP

- [ ] Rename support
- [ ] Find References
- [ ] Document Symbols / Outline
- [ ] Semantic Tokens (syntax highlighting from server)
- [ ] Code Actions (quick fixes for diagnostics)
- [ ] Context-aware completion (currently hardcoded keyword list)
- [ ] Incremental compilation (currently recompiles full workspace on every change)

## Parser & DSL

- [ ] Nested blocks (sub-entities or grouped properties)
- [ ] Entity inheritance / traits (extend an existing entity kind)
- [ ] Inline relationships (e.g. `Kael (leader of the Order)`)
- [ ] Implement `ww-dsl/src/resolver.rs` (cross-file name resolution — currently a stub)

## CLI

- [ ] Typed error enum for CLI instead of `Result<(), String>`
- [ ] `ww watch` — rebuild on file changes
- [ ] `ww diff` — compare two world states

## TUI

- [ ] Sort options (by name, kind, date)
- [ ] Export from within TUI
- [ ] Entity editing / creation
- [ ] Transitive relationship display in graph view

## Export

- [ ] CSV / TSV format
- [ ] YAML format
- [ ] Graphviz DOT format for relationship graphs

## VS Code Extension

- [ ] Snippet support (entity templates)
- [ ] Bracket matching configuration for `{ }` and `[ ]`
- [ ] Custom commands in command palette

## Example Project

- [ ] Expand `iron-kingdoms/` with more characters, deeper relationships, complex dates
- [ ] Add a second example world demonstrating custom entity kinds

## Hardening

- [ ] Audit `.unwrap()` in LSP server (`server.rs:81` — can panic on malformed URL)
- [ ] Replace `unwrap_or("???")` patterns with proper error propagation
