# Testing Skill

Use this skill when adding or improving tests, test fixtures, validation
commands, or regression coverage.

## Operating Rules

- Test behavior through public APIs where practical.
- Keep tests deterministic and independent.
- Use temporary directories or ephemeral ports for filesystem and network tests.
- Cover error paths, not only happy paths.
- Avoid sleeps and timing assumptions unless there is no reasonable alternative.

## Required Checks

- Run the smallest relevant test first while developing.
- Run the full workspace test command before finishing broad changes.
- Keep test failure messages useful for future contributors.

## Review Notes

Summaries should describe what behavior is covered, what remains untested, and
why any test gaps are acceptable.
