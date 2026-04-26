# codex-skils

codex-skils is a Rust CLI for creating and checking repository instructions,
engineering skill templates, and contributor-ready project structure.

It is intentionally small: templates are embedded in the binary, generated files
are explicit, and checks report exactly what is valid or missing.

## Quick Start

```bash
cargo run -p codex-sk-cli --bin codex-skils -- init
cargo run -p codex-sk-cli --bin codex-skils -- skill rust --write
cargo run -p codex-sk-cli --bin codex-skils -- check
```

`init` creates `.codex-skils/`, `.codex-skils/skills/`,
`.codex-skils/config.toml`, and `AGENTS.md`. Existing files are skipped unless
`--force` is passed.

## Commands

```bash
codex-skils --version
codex-skils init [--force]
codex-skils skill <name>
codex-skils skill <name> --write [--force]
codex-skils check
codex-skils health check
```

During local development, run the binary through Cargo:

```bash
cargo run -p codex-sk-cli --bin codex-skils -- --help
```

## Available Skills

- `rust`: Rust crate, CLI, runtime, and protocol engineering.
- `python`: Python SDK and developer-facing API work.
- `opensource`: contributor workflow and maintainer documentation.
- `devops`: CI, release checks, scripts, and automation.
- `security`: validation, secret handling, and security-sensitive changes.
- `testing`: test strategy, fixtures, and regression coverage.

By default, `skill <name>` prints the template to standard output.

```bash
codex-skils skill security
```

Use `--write` to create `.codex-skils/skills/<name>.md`.

```bash
codex-skils skill testing --write
codex-skils skill testing --write --force
```

## Generated File Structure

```text
.
├── AGENTS.md
└── .codex-skils/
    ├── config.toml
    └── skills/
        └── rust.md
```

The config file is intentionally simple:

```toml
schema_version = 1
project_name = "example"
default_skills = ["rust", "python", "opensource"]
```

## Repository Layout

```text
crates/
  core/        Core engine primitives
  protocol/    Shared types and errors
  runtime/     Runtime orchestration and local HTTP forwarding
  cli/         codex-skils command implementation
bindings/
  python/      Python SDK skeleton
templates/     Embedded skill templates
docs/
  architecture/
  rfcs/
```

## Architecture

The project is organized around explicit Rust crate boundaries and a Python
access layer. See [docs/architecture/overview.md](docs/architecture/overview.md)
for architecture goals, ownership model, API philosophy, testing approach, and
v0.1 scope.

## Development

Prerequisites:

- Rust 1.76 or newer
- Python 3.9 or newer

Rust checks:

```bash
cargo fmt --all --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Python checks:

```bash
python -m pip install -e bindings/python
python -m pytest bindings/python/tests
```

## Local HTTP Forwarder

The CLI also includes a minimal local HTTP forwarder:

```bash
codex-skils start-server --listen-port 8080 --target-port 9000
```

`start-server` listens on `127.0.0.1:<listen-port>` and forwards HTTP requests
to `127.0.0.1:<target-port>`.

## Project Principles

- Correctness first.
- Reliability second.
- Performance third.
- Developer experience always matters.
- Public APIs should be documented.
- Major design decisions should be captured in docs or RFCs.
