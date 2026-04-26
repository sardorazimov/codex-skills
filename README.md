# codex-sk

codex-sk is an open-source infrastructure project organized around a Rust core
and Python developer access layer.

The repository is intentionally early. The current goal is to establish a clean,
reviewable project structure before adding complex runtime behavior.

## Repository Layout

```text
crates/
  core/        Core engine primitives
  protocol/    Protocol types and validation boundaries
  runtime/     Runtime orchestration
  cli/         Command-line entry point
bindings/
  python/      Python package skeleton
docs/
  architecture/
  rfcs/
examples/
tests/
benchmarks/
scripts/
```

## Architecture

The project is organized around explicit Rust crate boundaries and a Python
access layer. See [docs/architecture/overview.md](docs/architecture/overview.md)
for the initial architecture goals, ownership model, API philosophy, testing
approach, and v0.1 scope.

## Prerequisites

- Rust 1.76 or newer
- Python 3.9 or newer

## Rust Development

```bash
cargo fmt --all
cargo test --workspace
cargo run -p codex-sk-cli
```

## Python Development

```bash
python -m pip install -e bindings/python
python -m pytest bindings/python/tests
```

## Project Principles

- Correctness first.
- Reliability second.
- Performance third.
- Developer experience always matters.
- Public APIs should be documented.
- Major design decisions should be captured in docs or RFCs.

## Status

This project is a skeleton and does not yet provide production runtime behavior.
See [ROADMAP.md](ROADMAP.md) for planned milestones.
