# Security Skill

Use this skill when changing input validation, error handling, dependency
policy, secret handling, reporting guidance, or security-sensitive behavior.

## Operating Rules

- Treat external input as untrusted.
- Return clear errors without leaking secrets or sensitive local paths.
- Prefer explicit allowlists for supported commands and formats.
- Keep security reports out of public issues when exploit details are involved.
- Do not add credentials, tokens, private keys, or production data to examples.

## Required Checks

- Test invalid input and failure paths.
- Review error messages for clarity and unnecessary disclosure.
- Check documentation for safe reporting and handling guidance.

## Review Notes

Summaries should identify the threat model, failure behavior, and residual risk.
