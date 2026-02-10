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
  - Solo: Solo TTRPG runner with Mythic GME-inspired oracle, scene management, and mechanics integration
  - Sheet: Character sheet viewer with attributes, skills, and tracks
  - Dice: Visual dice roller with customizable pools
- **Game Mechanics**: DSL-configurable TTRPG systems with dice, resolution strategies, character sheets, and combat
  - Presets: 2d20 (Modiphius), Trophy Gold, Blood & Honor, Mothership (d100 roll-under)
- **World Simulation**: Tick-based NPC schedules, needs, and spatial movement
- **Language Server**: Full LSP support with diagnostics, completion, go-to-definition, rename, and find references
- **Interactive Fiction**: Natural language parser, dialogue trees, narrator with configurable tone

### Example Worlds

Three example worlds are included:

- **iron-kingdoms/**: High fantasy setting demonstrating core features
- **stellar-drift/**: Sci-fi setting with custom entity kinds
- **thausand-empty-light/**: *A Thousand Empty Light* solo horror module for Mothership 1e
  - Fully configured Mothership mechanics (d100 roll-under, Stress/Health/Wounds tracks)
  - Authentic Semiotic Standard oracle (50 visual symbols) matching TEL's original design
  - Chaos/scene management disabled to preserve TEL's ORACLE workflow (Observe, Resolve, Act, Conclude, Leave Evidence)

## The DSL

DSL lexer, parser, and compiler for Weltenwanderer world files.

`.ww` files are the source of truth. They're human-readable, git-friendly,
and designed to feel natural while remaining strictly parseable.

### Entity Declarations

Every entity follows the pattern `<Name> is a <kind> { ... }`:

```ww
-- This is a comment

world "The Iron Kingdoms" {
    genre "high fantasy"
    setting "A shattered continent rebuilding after the Sundering"
}

the Iron Citadel is a fortress {
    climate arid
    population 45000
    north to the Ashlands

    """
    An ancient fortress carved from a single mountain of iron ore.
    Its walls have never been breached in three thousand years.
    """
}

Kael Stormborn is a character {
    species human
    occupation knight
    status alive
    traits [brave, stubborn, loyal]
    member of the Order of Dawn
    located at the Iron Citadel
    allied with Elara Nightwhisper
}

the Order of Dawn is a faction {
    type military_order
    led by Kael Stormborn
    based at the Iron Citadel
    values [honor, duty, sacrifice]
}

the Great Sundering is an event {
    date year -1247, month 3, day 15, era "Age of Ruin"
    type cataclysm
    involving [Kael Stormborn, the Order of Dawn]

    """
    The day the world broke.
    """
}

the Blade of First Light is an item {
    type weapon
    rarity legendary
    owned by Kael Stormborn
}

the Prophecy of Renewal is lore {
    type prophecy
    source "Ancient Dwarven Tablets"
    references [the Great Sundering, the Iron Citadel]
}
```

### Syntax Reference

| Pattern | Meaning |
|---|---|
| `<Name> is a <kind> { ... }` | Entity declaration |
| `<Name> is <kind> { ... }` | Entity declaration (no article) |
| `<key> <value>` | Property assignment |
| `<key> [a, b, c]` | List property |
| `<direction> to <Entity>` | Exit/connection (north, south, east, west, up, down) |
| `member of <Entity>` | Relationship: membership |
| `located at <Entity>` | Relationship: location |
| `allied with <Entity>` | Relationship: alliance |
| `rival of <Entity>` | Relationship: rivalry |
| `led by <Entity>` | Relationship: leadership |
| `owned by <Entity>` | Relationship: ownership |
| `based at <Entity>` | Relationship: headquarters |
| `in <Entity>` | Relationship: containment |
| `involving [<Entity>, ...]` | Relationship: participation |
| `references [<Entity>, ...]` | Relationship: reference |
| `caused by <Entity>` | Relationship: causation |
| `date year N, month N, day N, era "E"` | Date (year required, month/day/era optional) |
| `"""..."""` | Multiline description (Markdown) |
| `-- comment` | Line comment |
| `"string"` | Quoted string value |
| `45_000`, `-1247` | Numbers (Rust-style underscores allowed) |

### Entity Kinds

Built-in: `location`, `character`, `faction`, `event`, `item`, `lore`

Location subtypes (compiled as `location` with a subtype): `fortress`, `city`,
`town`, `village`, `region`, `continent`, `room`, `wilderness`, `dungeon`,
`building`, `landmark`, `plane`

Any unrecognized kind becomes a custom type.

### Multi-File Support

The compiler reads all `.ww` files in a directory. File boundaries don't
matter — entities can reference each other across files. See `iron-kingdoms/`
for an example world split across multiple files.

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
