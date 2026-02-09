# TODO

## M1: World Building Core (current)

### ww-core

- [x] Entity, World, Relationship, Query, Timeline types
- [x] Large-world stress tests

### ww-dsl

- [x] Lexer, parser, compiler, diagnostics
- [x] Nested blocks (sub-entities or grouped properties)
- [x] Entity inheritance / traits (extend an existing entity kind)
- [x] Inline relationships (e.g. `Kael (leader of the Order)`)
- [x] Implement resolver (cross-file name resolution)
- [x] Malformed input recovery tests (`parse_lenient` API)

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

- [x] `ww-simulation` crate
- [x] Tick-based simulation loop
- [x] NPC needs and schedules
- [x] Physics / spatial simulation
- [x] `ww simulate` CLI command with colored output
- [x] DSL syntax for simulation config (schedule, needs, speed blocks)
- [x] CharacterComponent fields for simulation data
- [x] Simulation systems read from entity components instead of defaults

## M5: Interactive Fiction Engine

- [x] `ww-fiction` crate (58 tests)
- [x] Natural language parser for player input (verb synonyms, fuzzy matching)
- [x] Choice engine (Dialogue, Choice, Condition, Effect, ChoiceState)
- [x] Narrator system (4 tones, 2 perspectives, template registry)
- [x] `FictionSession` — interactive game loop with movement, inventory, talk
- [x] `FictionSystem` — simulation plugin for narrative generation
- [x] `ww play` CLI command
- [x] DSL `dialogue` / `choice` block syntax with conditions, effects, goto
- [x] `FictionComponent` on entities (DialogueData, ChoiceData)
- [x] Example dialogues for Kael Stormborn and Thrain Ironhand
- [ ] Choice selection and effect application in `ww play`
- [ ] Save/load session state
- [ ] TUI fiction view

## M6: Game Mechanics

- [x] `ww-mechanics` crate (80 tests)
- [x] Dice system (Die, DicePool, DiceTag, RollResult with aggregation)
- [x] Resolution strategies (CountSuccesses, HighestDie, SumPool)
- [x] Character sheets and tracks (from_entity reads mechanics.* properties)
- [x] Rules engine (RuleSet.from_world loads DSL-configured systems)
- [x] Three presets: 2d20 (Modiphius), Trophy Gold, Blood & Honor
- [x] Combat system (participants, zones, initiative, actions, event log)
- [ ] CLI commands (`ww roll`, `ww check`, `ww combat`)

## M7: Solo TTRPG Runner

- [x] `ww-solo` crate (93 tests)
- [x] Oracle tables (fate chart, random events, NPC reaction)
- [x] Journaling system (append, export markdown/text)
- [x] Scene management (chaos check, altered/interrupted scenes)
- [x] Thread and NPC tracking
- [x] `ww solo` CLI command with interactive REPL
- [x] Auto-populate NPC tracker from world characters (skip dead)
- [x] DSL syntax for custom oracle tables (`oracle { actions [...] subjects [...] }`)
- [ ] Save/load session state

## M8: Pixel Art GUI

- [x] `ww-gui` crate (macroquad, 480x270 virtual canvas, PICO-8 palette, 8x8 bitmap font)
- [x] Title screen with directory input and world loading
- [x] Explorer with entity list, detail panel, search
- [x] Relationship graph (force-directed layout, zoom/pan, hover tooltips, node click)
- [x] Chronological timeline view
- [x] Interactive fiction play and solo TTRPG session screens
- [x] Character sheet viewer and dice roller
- [x] Mouse click support throughout (tabs, list, graph nodes, buttons)
- [x] Key repeat and mouse scroll wheel for all scrollable panels
- [ ] Sprite animation and visual polish
- [ ] Save/load session state for fiction and solo screens
- [ ] In-app entity editing

## Real-World Scenarios

- [ ] Run "A Thousand Empty Light" for a friend via screen sharing with Weltenwanderer
