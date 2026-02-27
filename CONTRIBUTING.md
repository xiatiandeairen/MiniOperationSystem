# Contributing to MiniOS

MiniOS is a teaching operating system. Contributions that make OS concepts
more accessible to learners are especially welcome.

## Getting Started

```bash
git clone https://github.com/xiatiandeairen/MiniOperationSystem.git
cd MiniOperationSystem
cargo make run-gui    # Boot in QEMU with graphical display
cargo make ci          # Run fmt + clippy + tests
```

## Development Environment

- **Rust nightly** — managed by `rust-toolchain.toml`
- **QEMU** — `apt install qemu-system-x86` (or brew/pacman equivalent)
- **cargo-make** — `cargo install cargo-make`

## How to Contribute

### Add a Shell Command

1. Create a handler function in `crates/shell/src/commands/`
2. Register it in `mod.rs` COMMANDS array
3. If it teaches an OS concept, add an `explain` entry
4. Run `cargo make ci` to verify

### Add an Experiment (Lab)

1. Add a function in `crates/shell/src/commands/lab.rs`
2. Include: setup → execution → observation → thinking question
3. Register in the `cmd_lab` match

### Fix a Bug

1. Check existing issues for the bug
2. Write a failing test if possible
3. Fix and verify with `cargo make ci`

## Code Style

- All kernel code is `#![no_std]`
- Functions ≤ 50 lines
- Public items need `///` doc comments
- `unsafe` blocks need `// SAFETY:` comments
- Commit format: `<type>(<scope>): <what it solves>`

## Architecture

See [spec.md](spec.md) for the full specification. Key rule:
each crate depends only on trait interfaces from `minios-common`,
never on concrete implementations from other crates.

## Testing

```bash
cargo make ci                    # Full CI pipeline
cargo make test                  # Unit tests only
cargo make run                   # Boot test (headless)
```

## Communication

- **Issues**: Bug reports, feature requests, questions
- **Pull Requests**: Code contributions
- **Discussions**: Ideas, learning experiences, feedback
