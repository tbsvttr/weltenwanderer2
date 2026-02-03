# TODO

## M1: World Building Core (current)

### ww-core

- [x] Entity, World, Relationship, Query, Timeline types
- [x] Large-world stress tests

### ww-dsl

- [x] Lexer, parser, compiler, diagnostics
- [ ] Nested blocks (sub-entities or grouped properties)
- [ ] Entity inheritance / traits (extend an existing entity kind)
- [ ] Inline relationships (e.g. `Kael (leader of the Order)`)
- [x] Implement resolver (cross-file name resolution)
- [ ] Malformed input recovery tests

### ww-cli

- [x] Commands: init, build, check, list, show, search, graph, timeline, export, new
- [x] ratatui TUI with entity list, detail, graph, timeline views
- [x] CLI command integration tests (10 commands, 28 tests)
- [ ] TUI app state and navigation tests
- [ ] Typed error enum instead of `Result<(), String>`
- [ ] `ww watch` — rebuild on file changes
- [ ] `ww diff` — compare two world states
- [ ] TUI: sort options (by name, kind, date)
- [ ] TUI: export from within TUI
- [ ] TUI: entity editing / creation
- [ ] TUI: transitive relationship display in graph view
- [ ] Export: CSV / TSV format
- [ ] Export: YAML format
- [ ] Export: Graphviz DOT format for relationship graphs

### ww-lsp

- [x] Diagnostics, go-to-definition, completion, hover
- [x] LSP server tests (53 tests)
- [x] Rename support (prepare + rename via AST reference search)
- [x] Find References (AST-based: definitions, relationships, exits)
- [x] Document Symbols / Outline (entity + world declarations)
- [x] Semantic Tokens (keyword, type, property, string, number, comment, operator)
- [x] Code Actions (quick fix: create stub entity from "undefined entity" diagnostic)
- [x] Context-aware completion (entity kind, relationship target, property value, body)
- [x] Incremental compilation (source hash skip when unchanged)
- [x] Audit `.unwrap()` in server.rs:81 (can panic on malformed URL)
- [x] Replace `unwrap_or("???")` patterns with proper error propagation

### VS Code extension

- [x] Syntax highlighting, LSP client integration
- [x] Snippet support (entity templates)
- [x] Bracket matching configuration for `{ }` and `[ ]`
- [x] Custom commands in command palette

### Example project

- [x] iron-kingdoms/ — basic example world
- [x] Expand with more characters, deeper relationships, complex dates
- [x] Add a second example world demonstrating custom entity kinds (stellar-drift/)

### ww-server — not started

- [ ] Axum HTTP API crate
- [ ] CRUD endpoints for entities and relationships
- [ ] Search, graph, and timeline API endpoints
- [ ] Serve static web UI assets

### Web UI — not started

- [ ] React + Vite + TypeScript frontend
- [ ] React Flow graph visualization
- [ ] Markdown editor for entity descriptions
- [ ] TanStack Query for data fetching
- [ ] Dashboard with world overview

### ww-wasm — not started

- [ ] WebAssembly bindings for ww-core
- [ ] ts-rs for Rust-to-TypeScript type sharing

## M2: AI Integration

- [ ] `ww-ai` crate — provider abstraction
- [ ] Anthropic provider
- [ ] OpenAI provider
- [ ] Ollama provider (local models)

## M4: World Simulation

- [ ] `ww-simulation` crate
- [ ] Tick-based simulation loop
- [ ] NPC needs and schedules
- [ ] Physics / spatial simulation

## M5: Interactive Fiction Engine

- [ ] `ww-fiction` crate
- [ ] Natural language parser for player input
- [ ] Choice engine (branching narratives)
- [ ] Narrator system

## M6: Game Mechanics

- [ ] `ww-mechanics` crate
- [ ] Dice system and probability
- [ ] Character sheets and stats
- [ ] Rules engine
- [ ] Combat system

## M7: Solo TTRPG Runner

- [ ] `ww-solo` crate
- [ ] Oracle tables (yes/no, event, NPC reaction)
- [ ] Journaling system
- [ ] Scene management
