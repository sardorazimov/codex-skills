# DevOps Skill

Use this skill when changing CI, release checks, local automation, repository
scripts, or operational documentation.

## Operating Rules

- Keep workflows simple, deterministic, and readable.
- Avoid heavyweight services unless the project already depends on them.
- Cache only when it is clear and safe.
- Make failure output actionable for contributors.
- Keep release checks stricter than local convenience scripts.

## Required Checks

- Validate workflow commands locally when possible.
- Ensure CI paths match the files they are meant to protect.
- Confirm scripts do not require secrets for normal validation.

## Review Notes

Summaries should include workflow names, changed commands, expected triggers,
and any environment assumptions.
