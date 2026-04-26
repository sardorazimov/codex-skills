# AGENTS.md

## Project identity

This is a serious open-source infrastructure project.

The repository must feel:
- production-grade
- contributor-friendly
- technically respected
- easy to run locally
- easy to review in pull requests

## Architecture

- Rust is used for the core engine, protocol, runtime, and CLI.
- Python is used for bindings, SDK, examples, and developer accessibility.
- Keep crates separated by responsibility.
- Do not mix protocol, runtime, CLI, and SDK logic.
- Public APIs must be documented.
- Major design decisions must be explained in docs or RFCs.

## Engineering rules

- Correctness first.
- Reliability second.
- Performance third.
- Developer experience always matters.
- Prefer simple explicit code over clever abstractions.
- No placeholders.
- No fake TODOs.
- No fake production claims.
- No unnecessary dependencies.
- No secrets in code.
- Validate external input.
- Return complete working files, not fragments.

## Testing

Before finishing a task, run the relevant checks when possible:

- Rust format
- Rust tests
- Python format/checks if Python was changed
- Documentation links/checks if docs were changed

If a check cannot be run, explain why.

## Output rules

For every task:
- summarize what changed
- list files changed
- list commands run
- list remaining risks