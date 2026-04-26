# codex-sk Python SDK

This package provides the initial Python SDK surface for codex-sk.

The current SDK is intentionally local and does not require Rust FFI. It exposes
a typed client, typed models, and clear exceptions while the Rust runtime and
protocol surfaces stabilize.

## Usage

```python
from codex_sk import CodexSkClient

client = CodexSkClient()

print(client.version())

report = client.health_check()
print(report.component, report.status.value, report.detail)
```

## Public API

- `CodexSkClient`: local SDK client.
- `PackageInfo`: package metadata.
- `HealthReport`: health check result.
- `HealthStatus`: health status enum.
- `CodexSkError`: base SDK exception.
- `HealthCheckError`: raised when a health check reports an unhealthy component.

## Development

```bash
python -m pip install -e bindings/python
python -m pytest bindings/python/tests
```
