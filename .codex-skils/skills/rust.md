# Rust Engineering Skill

Use this skill when changing Rust crates, command-line behavior, runtime code,
protocol types, or public Rust APIs.

## Operating Rules

- Keep crate boundaries explicit and documented.
- Prefer typed data and typed errors over string-based control flow.
- Validate external input at the protocol or CLI boundary.
- Keep public APIs small, documented, and covered by tests.
- Avoid new dependencies unless they clearly reduce risk or maintenance cost.

## Required Checks

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`

## Review Notes

Summaries should identify changed crates, public API impact, commands run, and
remaining risks.
