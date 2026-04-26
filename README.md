# codex-skils

Structured AI/Codex engineering rules for serious developer repositories.

codex-skils is a Rust + Python CLI tool for managing project-level engineering
instructions through reusable skill files. It helps teams keep `AGENTS.md`,
repository rules, and AI-assisted development guidance consistent without
copying large blocks of text by hand.

The tool exists because AI/Codex workflows work best when repository rules are
explicit, versioned, reviewable, and easy to regenerate.

## Features

- Bootstrap project rule files with `init`.
- Generate production-oriented skill templates with `skill`.
- Merge local skills into `AGENTS.md` with an idempotent `apply` command.
- List available built-in skills with descriptions.
- Export skills as Markdown, JSON, or YAML.
- Validate required project structure with `check`.
- Generate files safely: existing files are not overwritten unless `--force` is used.
- Preserve user-written content outside managed sections.

## Quick Start

From this repository:

```bash
cargo run -p codex-sk-cli --bin codex-skils -- init
cargo run -p codex-sk-cli --bin codex-skils -- skill rust --write
cargo run -p codex-sk-cli --bin codex-skils -- apply --dry-run
cargo run -p codex-sk-cli --bin codex-skils -- apply
cargo run -p codex-sk-cli --bin codex-skils -- check
```

Example output:

```text
apply complete
found 1 skill(s)
updated AGENTS.md (managed section added)
```

## Example Workflow

Start with an existing repository that has a `README.md` but no Codex rule
system.

1. Initialize codex-skils:

   ```bash
   codex-skils init
   ```

   This creates:

   ```text
   AGENTS.md
   .codex-skils/
   .codex-skils/config.toml
   .codex-skils/skills/
   ```

2. Add skills:

   ```bash
   codex-skils skill rust --write
   codex-skils skill security --write
   codex-skils skill testing --write
   ```

3. Preview the merge:

   ```bash
   codex-skils apply --dry-run --readme
   ```

   Example output:

   ```text
   apply complete
   found 3 skill(s)
   would update AGENTS.md (managed section added)
   would update README.md (managed section added)
   dry run: no files changed
   ```

4. Apply the rules:

   ```bash
   codex-skils apply --readme
   ```

5. Validate the project:

   ```bash
   codex-skils check
   ```

## CLI Usage

### `init`

Bootstrap codex-skils files.

```bash
codex-skils init
codex-skils init --force
```

`init` creates `AGENTS.md`, `.codex-skils/`, `.codex-skils/skills/`, and
`.codex-skils/config.toml`. Existing files are skipped unless `--force` is
provided.

### `skill`

Print or write a built-in skill template.

```bash
codex-skils skill rust
codex-skils skill python --format json
codex-skils skill security --format yaml
codex-skils skill testing --write
codex-skils skill testing --write --force
```

By default, skills are printed as Markdown. `--write` saves the skill to
`.codex-skils/skills/<name>.md`.

### `apply`

Merge local skill files into project documentation.

```bash
codex-skils apply
codex-skils apply --dry-run
codex-skils apply --force
codex-skils apply --readme
codex-skils apply --readme --force
```

`apply` is idempotent. Re-running it does not duplicate skill content.

### `list`

Show all built-in skills and short descriptions.

```bash
codex-skils list
```

Example output:

```text
available skills
rust - Rust crate, CLI, runtime, and protocol engineering.
python - Python SDK and developer-facing API work.
opensource - Contributor workflow and maintainer documentation.
devops - CI, release checks, scripts, and automation.
security - Validation, secret handling, and security-sensitive changes.
testing - Test strategy, fixtures, and regression coverage.
```

### `export`

Export built-in skills.

```bash
codex-skils export --all
codex-skils export --all --format json
codex-skils export --all --format yaml
codex-skils export --all --output .codex-skils/export
codex-skils export --all --format json --output .codex-skils/export
```

Markdown export writes one file per skill when `--output` is used. JSON and YAML
export write `skills.json` or `skills.yaml`.

### `check`

Validate the expected project structure.

```bash
codex-skils check
```

Example output:

```text
check passed
valid README.md
valid AGENTS.md
valid CONTRIBUTING.md
valid SECURITY.md
valid .codex-skils/config.toml
valid .codex-skils/skills
```

## Available Skills

- `rust`: Rust crate, CLI, runtime, and protocol engineering.
- `python`: Python SDK and developer-facing API work.
- `opensource`: contributor workflow and maintainer documentation.
- `devops`: CI, release checks, scripts, and automation.
- `security`: validation, secret handling, and security-sensitive changes.
- `testing`: test strategy, fixtures, and regression coverage.

## How Apply Works

`codex-skils apply` reads Markdown files from `.codex-skils/skills/*.md`, sorts
them alphabetically, and merges them into `AGENTS.md` inside a managed section:

```md
<!-- codex-skils:start -->
## Skills

... generated content ...

<!-- codex-skils:end -->
```

Only the managed section is replaced. Content outside the markers is preserved.
If `AGENTS.md` does not exist, it is created. If managed markers are malformed,
the command fails with a clear error unless `--force` is used.

With `--readme`, codex-skils also manages a `README.md` section:

```md
<!-- codex-skils:readme:start -->
## Development Rules

This project uses codex-skils to manage AI/Codex engineering rules.

Active skills:

- README
- rust

<!-- codex-skils:readme:end -->
```

Use `--dry-run` before applying changes in a repository with existing rules.

## Project Structure

```text
.
├── AGENTS.md
├── README.md
└── .codex-skils/
    ├── config.toml
    └── skills/
        ├── rust.md
        ├── security.md
        └── testing.md
```

Configuration is intentionally small:

```toml
schema_version = 1
project_name = "codex-skils"
default_skills = ["rust", "python", "opensource"]
```

## Development

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

## Contributing

Contributions should be small, reviewed, and tested. Before opening a pull
request, run the relevant checks and include the commands in the PR description.
See `CONTRIBUTING.md` for project guidelines.

## License

Licensed under the Apache License, Version 2.0. See `LICENSE` for details.
