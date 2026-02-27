# docs/ — Documentation Index

## Key Documents

| Document | Description |
|----------|-------------|
| [../spec.md](../spec.md) | Full technical specification |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | Contributor guide |
| [../CHANGELOG.md](../CHANGELOG.md) | Release changelog |
| [comparison.md](comparison.md) | MiniOS vs Linux design comparison |
| [user-testing-guide.md](user-testing-guide.md) | How to test MiniOS as a learner |
| [web-integration-guide.md](web-integration-guide.md) | Web-based integration notes |

## Directory Structure

```
docs/
├── README.md                ← This file: documentation index
├── comparison.md            ← MiniOS vs Linux design comparison
├── user-testing-guide.md    ← Learner testing guide
├── web-integration-guide.md ← Web integration notes
├── releases/                ← Version release notes
│   ├── v0.1.0.md … v0.7.0.md
│   └── v2.0.0-vision.md
├── changelogs/              ← Detailed per-version changelogs
│   └── v0.1.0 … v0.7.0
├── dev-logs/                ← AI coding task logs and insights
│   └── v0.1.0 … v0.7.0 (tasks + insights)
├── audit/                   ← Code quality audits
│   └── v2.2.0-full-audit.md
├── tasks/                   ← Iteration planning
│   ├── v08-v27-iteration-plan.md
│   └── v28-v47-iteration-plan.md
└── adr/                     ← Architecture Decision Records
    ├── 001-use-rust-nightly.md
    ├── 002-bootloader-api-crate.md
    └── 003-multi-crate-workspace.md
```

## Architecture Decision Records (ADRs)

| ADR | Decision |
|-----|----------|
| [001](adr/001-use-rust-nightly.md) | Use Rust nightly for bare-metal features |
| [002](adr/002-bootloader-api-crate.md) | Use `bootloader_api` crate for boot protocol |
| [003](adr/003-multi-crate-workspace.md) | Multi-crate workspace architecture |
