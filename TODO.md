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
- [x] TUI app state and navigation tests (moved to ww-tui crate)
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

- [x] `ww-fiction` crate (68 tests)
- [x] Natural language parser for player input (verb synonyms, fuzzy matching)
- [x] Choice engine (Dialogue, Choice, Condition, Effect, ChoiceState)
- [x] Narrator system (4 tones, 2 perspectives, template registry)
- [x] `FictionSession` — interactive game loop with movement, inventory, talk
- [x] `FictionSystem` — simulation plugin for narrative generation
- [x] `ww play` CLI command
- [x] DSL `dialogue` / `choice` block syntax with conditions, effects, goto
- [x] `FictionComponent` on entities (DialogueData, ChoiceData)
- [x] Example dialogues for Kael Stormborn and Thrain Ironhand
- [x] Narrator wired into FictionSession (tone-aware descriptions, arrivals, movement)
- [x] Fiction config from DSL (`fiction.tone`, `fiction.start`, `fiction.perspective`)
- [ ] Choice selection and effect application in `ww play`
- [ ] Save/load session state
- [ ] TUI fiction view

## M6: Game Mechanics

- [x] `ww-mechanics` crate (99 tests)
- [x] Dice system (Die, DicePool, DiceTag, RollResult with aggregation)
- [x] Resolution strategies (CountSuccesses, HighestDie, SumPool, RollUnder)
- [x] Character sheets and tracks (from_entity reads mechanics.* properties)
- [x] Rules engine (RuleSet.from_world loads DSL-configured systems)
- [x] Four presets: 2d20 (Modiphius), Trophy Gold, Blood & Honor, Mothership (d100 roll-under)
- [x] Combat system (participants, zones, initiative, actions, event log)
- [ ] CLI commands (`ww roll`, `ww check`, `ww combat`)

## M7: Solo TTRPG Runner

- [x] `ww-solo` crate (156 tests)
- [x] Oracle tables (fate chart, random events, NPC reaction)
- [x] Journaling system (append, export markdown/text)
- [x] Scene management (chaos check, altered/interrupted scenes)
- [x] Thread and NPC tracking
- [x] `ww solo` ratatui TUI with action buttons, tab completion, track gauges
- [x] Auto-populate NPC tracker from world characters (skip dead)
- [x] DSL syntax for custom oracle tables (`oracle { actions [...] subjects [...] }`)
- [x] World-driven solo config (scene text, chaos label, event/reaction prefixes)
- [x] Mechanics integration (check, roll, sheet, panic, encounter commands)
- [x] `panic` command (d20 vs Stress with auto-increment)
- [x] `encounter <creature>` command (display creature stats from world)
- [x] Context-aware tab completion (`completions()` API)
- [ ] Save/load session state

## M8: Terminal UI (Unified)

- [x] `ww-tui` crate (ratatui 0.29, crossterm 0.28, standalone binary, 37 tests)
- [x] Tab trait with InputMode (VimNav vs TextInput) for unified event routing
- [x] Explorer tab (entity list/detail, search, vim-like navigation)
- [x] Graph tab (ASCII relationship view, scrollable)
- [x] Timeline tab (chronological events, selectable)
- [x] Play tab (interactive fiction with text input, output scroll)
- [x] Solo tab (TTRPG with action buttons, tab completion, sidebar with tracks/chaos/threads/NPCs)
- [x] Sheet tab (character sheet viewer with track gauges, two-column attributes/skills)
- [x] Dice tab (visual dice roller with die type selector and pool size controls)
- [x] Mouse support (click tabs, action buttons, scroll output)
- [x] Context-aware status bar per tab
- [x] Lazy initialization for play/solo tabs
- [x] CLI delegation (`ww tui`, `ww play`, `ww solo` launch `ww-tui` binary)
- [x] Replaced ww-gui (removed macroquad pixel art GUI)
- [ ] Save/load session state for play and solo tabs
- [ ] In-app entity editing

## Real-World Scenarios

- [x] Run "A Thousand Empty Light" for a friend via screen sharing
  - [x] DSL parser supports numbers in entity names (Section 1, UPB 154, TEL 022)
  - [x] Play and Solo tabs reachable via unified TUI (Ctrl+4, Ctrl+5)
  - [x] Welcome text / onboarding when entering Play and Solo screens
  - [x] Play status bar mentions `help` command
  - [x] Solo screen shows intro text from DSL config
  - [x] World-driven fiction config (narrator tone, start location, perspective)
  - [x] World-driven solo config (scene text, chaos label, event/reaction prefixes)
  - [x] TUI sidebar uses world config (chaos_label from DSL)
  - [x] `panic` command — d20 vs Stress with auto-increment
  - [x] `encounter <creature>` command — display creature stats from world
  - [x] Unified TUI with tab bar, action buttons, tab completion, track gauges
  - [x] Comprehensive test coverage for TUI mouse/keyboard interaction (37 tests for explorer tab)
  - **TEL Compatibility Note**: Mothership 1e mechanics fully supported. Oracle is adapted (action/subject tables vs TEL's original Semiotic Standard d50 visual symbols). Chaos/pressure/scene system is an added feature for extended solo play, not present in original TEL workflow (Observe, Resolve, Act, Conclude, Leave Evidence).
  - [ ] Save/load session state for fiction and solo screens
