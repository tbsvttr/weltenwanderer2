<!-- Generated file — do not edit. Edit doc/README.tmpl.md or crate doc comments instead. -->
# Weltenwanderer

A creative engine for world building, powered by a custom DSL. Define worlds, characters, factions, locations, events, items, and lore in plain-text `.ww` files — then compile, query, explore, and export them.

## Quick Start

```bash
cargo build --release

ww init "My World"
cd "My World"
# edit world.ww, then:
ww build
ww tui
```

Run `ww --help` for all commands and options.

## Features

- **Terminal UI** (`ww tui`): Tabbed ratatui interface with 7 modes:
  - Explorer: Entity list and detail viewer with search
  - Graph: ASCII relationship visualization
  - Timeline: Chronological event browser
  - Play: Interactive fiction session with natural language parser
  - Solo: Solo TTRPG runner with Mythic GME-inspired oracle, optional scene management, and mechanics integration
  - Sheet: Character sheet viewer with attributes, skills, and tracks
  - Dice: Visual dice roller with customizable pools
- **Game Mechanics**: DSL-configurable TTRPG systems with dice, resolution strategies, character sheets, and combat
  - Presets: 2d20 (Modiphius), Trophy Gold, Blood & Honor, Mothership (d100 roll-under)
- **World Simulation**: Tick-based NPC schedules, needs, and spatial movement
- **Language Server**: Full LSP support with diagnostics, completion, go-to-definition, rename, and find references
- **Interactive Fiction**: Natural language parser, dialogue trees, location-proximate talk, narrator with configurable tone

### Example Worlds

Three example worlds are included:

- **iron-kingdoms/**: High fantasy setting demonstrating core features
- **stellar-drift/**: Sci-fi setting with custom entity kinds
- **thausand-empty-light/**: *A Thousand Empty Light* solo horror module for Mothership 1e
  - Fully configured Mothership mechanics (d100 roll-under, Stress/Health/Wounds tracks)
  - Authentic Semiotic Standard oracle (50 visual symbols) matching TEL's original design
  - Chaos/scene management disabled to preserve TEL's ORACLE workflow (Observe, Resolve, Act, Conclude, Leave Evidence)
  - MemoComm recordings discovered per section as you descend (location-proximate talk)

## The DSL

{{DSL_REFERENCE}}

## Building

Requires Rust 1.85+ (edition 2024).

```bash
cargo build --release
# Binaries: target/release/ww, target/release/ww-lsp, target/release/ww-tui
```

A VS Code extension for syntax highlighting and LSP integration is in `editors/vscode/`.

## Development

```bash
make setup    # install git hooks + cargo-deny (run once)
make check    # run all quality checks
make fix      # auto-fix formatting + clippy suggestions
```

A pre-commit hook enforces formatting, clippy, tests, documentation, and dependency auditing on every commit. See the [Makefile](Makefile) for all targets.

## License

MIT OR Apache-2.0
