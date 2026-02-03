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

## The DSL

{{DSL_REFERENCE}}

## Building

Requires Rust 1.85+ (edition 2024).

```bash
cargo build --release
# Binaries: target/release/ww, target/release/ww-lsp
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
