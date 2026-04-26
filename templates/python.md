# Python SDK Skill

Use this skill when changing Python package layout, SDK clients, examples,
typing, or developer-facing Python APIs.

## Operating Rules

- Keep public APIs typed and documented.
- Use clear exceptions for expected failure modes.
- Keep examples runnable from a clean checkout.
- Avoid Rust FFI until the Rust boundary is stable and necessary.
- Do not duplicate behavior that belongs in the Rust protocol or runtime.

## Required Checks

- Install the package in editable mode when tests need imports.
- Run Python tests for changed SDK behavior.
- Run the minimal example when example behavior changes.

## Review Notes

Summaries should identify public API changes, compatibility risks, and commands
run locally.
