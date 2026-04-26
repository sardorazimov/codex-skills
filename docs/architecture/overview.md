# Architecture Overview

This document describes the intended architecture for codex-skils v0.1. The goal
is to keep the early project understandable, reviewable, and ready for careful
growth without implying that production runtime behavior exists before it does.

## Project Goals

codex-skils is a Rust-first infrastructure project with a Python access layer. The
project should provide a reliable core, explicit protocol boundaries, and
developer tooling that can be understood by contributors without relying on
hidden service assumptions.

For v0.1, the primary goals are:

- establish clean crate boundaries
- define where public APIs will live
- make local development and review straightforward
- document major design decisions before they become implicit behavior
- keep examples honest and runnable as functionality is added

The project should prefer small, explicit components over broad abstractions.
New behavior should be introduced only when its ownership, validation boundary,
and testing strategy are clear.

## Crate Responsibilities

The Rust workspace is split by responsibility.

### `crates/core`

`core` owns domain primitives and engine-level concepts that are independent of
transport, orchestration, CLI UX, and language bindings.

Code in this crate should be deterministic where practical and should avoid
environment-specific assumptions. It should not parse CLI arguments, manage
process lifecycle, or expose Python-specific types.

### `crates/protocol`

`protocol` owns protocol types, validation rules, versioning policy, and
compatibility-sensitive data structures.

This crate is the boundary for external input. Runtime and CLI code should rely
on validated protocol types rather than duplicating ad hoc parsing rules.

### `crates/runtime`

`runtime` coordinates core behavior with protocol-level inputs. It owns
orchestration, lifecycle decisions, and runtime composition.

This crate should not define protocol schemas, command-line UX, or Python SDK
interfaces. It may depend on `core` and `protocol`.

### `crates/cli`

`cli` owns the command-line entry points and user-facing terminal behavior.

The CLI should parse arguments, generate repository files, load embedded skill
templates, call runtime APIs where needed, report clear results, and return
meaningful exit codes. Core behavior should remain in the lower-level crates.

### `templates`

`templates` contains the skill templates embedded into the `codex-skils`
binary. Keeping templates as files makes them reviewable while `include_str!`
keeps the installed CLI self-contained.

### `.codex-skils`

Generated project configuration lives under `.codex-skils/`. The initial config
file is TOML with a `schema_version`, `project_name`, and `default_skills`.
Generated skill files live under `.codex-skils/skills/`.

## Rust and Python Boundary

Rust is the source of truth for the core engine, protocol, runtime, and CLI.
Python exists to provide developer accessibility through bindings, SDK helpers,
and examples.

The Python package should not reimplement Rust behavior. When Python exposes a
feature backed by Rust, it should wrap stable Rust APIs and preserve the same
validation and error semantics where practical.

Python code may provide convenience helpers when they improve developer
experience, but those helpers should remain clearly separate from protocol and
runtime ownership. If a helper becomes essential behavior, it should be
evaluated for movement into Rust.

## Public API Philosophy

Public APIs should be intentional, documented, and small enough to support.

Before exposing a new public API, contributors should be able to answer:

- which crate or package owns it
- what inputs are accepted and validated
- what errors callers should expect
- whether the behavior is stable enough for downstream users
- how compatibility will be handled in future releases

Internal code can change freely while the project is young. Public APIs should
change more carefully, with migration notes when needed.

## Testing Philosophy

Tests should match the risk of the change.

Unit tests belong close to the Rust crate or Python package behavior they
exercise. Integration tests belong under `tests/integration` when behavior
crosses crate, process, protocol, or language boundaries.

For v0.1, the expected checks are:

- Rust formatting for Rust changes
- Rust clippy for CLI, runtime, and protocol changes
- Rust tests for workspace changes
- Python tests for Python package changes
- documentation review for architecture, RFC, and contributor-facing changes

Benchmarks should be added only after there is stable behavior worth measuring.
They should document what is measured and how results should be interpreted.

## Contributor Workflow

Contributors should start with the narrowest change that solves the problem.

A typical workflow is:

1. Read the relevant crate or documentation boundary.
2. Make a focused change.
3. Add or update tests when behavior changes.
4. Run the relevant local checks.
5. Open a pull request that explains motivation, scope, validation, and risks.

Changes that affect public APIs, architecture boundaries, compatibility, or
security posture should be documented in `docs/architecture` or proposed through
an RFC in `docs/rfcs`.

## Out of Scope for v0.1

The following are intentionally out of scope for v0.1:

- production runtime guarantees
- distributed system behavior
- stable external protocol compatibility promises
- native Python extension bindings
- package-manager installation flows
- performance tuning before stable behavior exists
- deployment automation
- claims of operational readiness

These areas may become important later, but v0.1 should first establish a
maintainable foundation with clear ownership and reliable local development.
