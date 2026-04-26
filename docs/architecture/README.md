# Architecture

codex-sk is organized around explicit ownership boundaries:

- `crates/core`: domain primitives and core engine concepts.
- `crates/protocol`: protocol types, validation rules, and compatibility policy.
- `crates/runtime`: orchestration of core behavior and protocol-level inputs.
- `crates/cli`: command-line UX and process entry points.
- `bindings/python`: Python bindings, SDK helpers, and accessibility examples.

The project should keep protocol, runtime, CLI, and SDK logic separate. When a
change crosses those boundaries, document the design in this directory or in an
RFC under `docs/rfcs`.

## Current State

This repository currently contains the initial skeleton only. It is suitable for
establishing build, review, and contribution workflows before deeper runtime
behavior is introduced.
