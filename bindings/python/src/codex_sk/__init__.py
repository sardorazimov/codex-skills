"""Public Python SDK surface for codex-sk."""

from codex_sk.client import CodexSkClient
from codex_sk.exceptions import CodexSkError, HealthCheckError
from codex_sk.models import HealthReport, HealthStatus, PackageInfo

__all__ = [
    "__version__",
    "CodexSkClient",
    "CodexSkError",
    "HealthCheckError",
    "HealthReport",
    "HealthStatus",
    "PackageInfo",
    "package_info",
]

__version__ = "0.1.0"


def package_info() -> PackageInfo:
    """Return package metadata for smoke tests and examples."""
    return PackageInfo(name="codex-sk", version=__version__)
