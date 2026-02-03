# Weltenwanderer

A creative engine for world building, powered by a custom DSL. Define worlds, characters, factions, locations, events, items, and lore in plain-text `.ww` files — then compile, query, explore, and export them.

## Quick Start

```bash
# Build the project
cargo build --release

# Create a new world
ww init "My World"
cd "My World"

# Edit world.ww, then:
ww build          # Compile and check for errors
ww list            # List all entities
ww show "Kael" -r  # Show entity with relationships
ww tui             # Launch interactive explorer
```

## The DSL

`.ww` files are the source of truth. They're human-readable, git-friendly, and designed to feel natural while remaining strictly parseable.

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
    date year -1247, month 3, day 15
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
| `date year N, month N, day N` | Date (year required, month/day optional) |
| `"""..."""` | Multiline description (Markdown) |
| `-- comment` | Line comment |
| `"string"` | Quoted string value |
| `45_000`, `-1247` | Numbers (Rust-style underscores allowed) |

### Entity Kinds

Built-in: `location`, `character`, `faction`, `event`, `item`, `lore`

Location subtypes (compiled as `location` with a subtype): `fortress`, `city`, `town`, `village`, `region`, `continent`, `room`, `wilderness`, `dungeon`, `building`, `landmark`, `plane`

Any unrecognized kind becomes a custom type.

### Multi-File Support

Split large worlds across files however you like:

```
my-world/
├── world.ww
├── locations.ww
├── characters.ww
├── factions.ww
├── history.ww
├── items.ww
└── lore.ww
```

The compiler reads all `.ww` files in a directory. File boundaries don't matter — entities can reference each other across files.

## CLI Commands

```
ww init <name>                     Create a new world directory with a template
ww build [-d <dir>]                Compile .ww files, report diagnostics
ww check [-d <dir>]                Validate without full build output
ww list [kind] [-t tag] [-d <dir>] List entities, optionally filtered
ww show <name> [-r] [-d <dir>]     Show entity detail (-r for relationships)
ww search <query> [-d <dir>]       Full-text search across names and descriptions
ww graph [-f <entity>] [-d <dir>]  ASCII relationship graph
ww timeline [--from Y] [--to Y]    Chronological event display
ww export <format> [-o path]       Export to json, markdown, or html
ww new <kind> <name> [-f file]     Generate a DSL stub and append to file
ww tui [-d <dir>]                  Launch interactive TUI explorer
ww lsp                             Start the Language Server Protocol server
```

All commands default to the current directory for `-d`.

## TUI

`ww tui` launches an interactive terminal explorer with three views:

| Key | Action |
|---|---|
| `j` / `k` or arrows | Navigate / scroll |
| `Enter` | Select entity or event |
| `Esc` | Go back |
| `/` | Search (filter by name) |
| `Tab` | Cycle views |
| `1` / `2` / `3` | Switch to Entities / Graph / Timeline |
| `?` | Toggle help |
| `q` | Quit |

## LSP

The `ww-lsp` binary provides IDE integration via the Language Server Protocol:

- Diagnostics (errors and warnings on save)
- Go-to-definition for entity references
- Autocomplete for entity names and keywords
- Hover to see entity kind

Point your editor's LSP client at `ww-lsp` for `.ww` files.

## Building

Requires Rust 1.88+.

```bash
cargo build --release

# Binaries:
#   target/release/ww       — CLI + TUI
#   target/release/ww-lsp   — Language server
```

## Project Structure

```
crates/
├── ww-core/     Core types: Entity, World, Relationship, Query, Timeline
├── ww-dsl/      DSL lexer (logos), parser (chumsky), compiler, diagnostics (ariadne)
├── ww-cli/      CLI commands + ratatui TUI
└── ww-lsp/      tower-lsp language server
```

## License

MIT OR Apache-2.0
