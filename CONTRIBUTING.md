# Contributing to MiniOS

Thank you for your interest in MiniOS!

## Quick Start

```bash
git clone <repo>
cargo make run-gui   # Boot and explore
cargo make ci         # Run all checks
```

## Development Workflow

1. Create a feature branch
2. Make changes (one feature per commit)
3. Run `cargo make ci` (fmt + clippy + test)
4. Submit a pull request

## Code Guidelines

- All code is `#![no_std]` (except trace-macros)
- Functions ≤ 50 lines
- Public items must have `///` doc comments
- `unsafe` code must have `// SAFETY:` comment
- Commit format: `<type>(<scope>): <what problem it solves>`

## Adding a Shell Command

1. Create handler in `crates/shell/src/commands/`
2. Register in `mod.rs` COMMANDS array
3. Add `explain` entry if it teaches an OS concept
4. Add `journey::mark()` if it's a learning step

## Architecture

See [spec.md](spec.md) for full technical specification.
Each subsystem is a separate crate communicating through traits in `minios-common`.
