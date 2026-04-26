# Contributing

Thank you for helping improve codex-sk.

This project values small, reviewable changes with clear motivation and reliable
validation. You do not need to understand the whole repository before
contributing; start with the smallest area related to your change.

## Ways to Contribute

- Improve documentation, examples, and setup instructions.
- Add focused tests for existing behavior.
- Fix small bugs with clear reproduction steps.
- Propose design changes through issues or RFCs before large implementation
  work.

Good first contributions should be narrow, easy to review, and safe to revert.

## Local Setup

Prerequisites:

- Rust 1.76 or newer
- Python 3.9 or newer

Useful commands:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

python -m pip install -e bindings/python
python -m pytest bindings/python/tests
```

If a command cannot run in your environment, mention that in the pull request
and include the error or reason.

## Pull Requests

Before opening a pull request:

- Keep changes scoped to one concern.
- Add or update tests when behavior changes.
- Document public APIs.
- Explain architecture-level decisions in `docs/architecture` or `docs/rfcs`.
- Run relevant formatters and tests.

Pull requests should include:

- what changed
- why it changed
- how it was tested
- known risks or follow-up work

Draft pull requests are welcome for early feedback. Mark the PR ready for review
when the scope is clear and the relevant checks have been run.

## Code Boundaries

- Keep core domain logic in `crates/core`.
- Keep protocol definitions and validation in `crates/protocol`.
- Keep orchestration in `crates/runtime`.
- Keep command-line behavior in `crates/cli`.
- Keep Python bindings and SDK behavior in `bindings/python`.

## Security

Do not include secrets, credentials, private keys, or sensitive production data
in issues, pull requests, examples, tests, or documentation.

Report vulnerabilities through the process in `SECURITY.md`, not in public
issues.

## Communication

Be direct, specific, and respectful. Assume good intent, ask clarifying
questions when needed, and prefer technical evidence over broad claims.
