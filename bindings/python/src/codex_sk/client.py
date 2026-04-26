"""Typed public client for local codex-sk SDK behavior."""

from __future__ import annotations

from codex_sk.exceptions import HealthCheckError
from codex_sk.models import HealthReport, HealthStatus, PackageInfo


class CodexSkClient:
    """Small local client for codex-sk SDK consumers.

    The client intentionally does not use Rust FFI yet. It provides the public
    Python API shape for local metadata and health behavior while the Rust
    runtime and protocol surfaces are still stabilizing.
    """

    def __init__(self, *, package_name: str = "codex-sk", version: str = "0.1.0") -> None:
        """Create a local SDK client."""
        if not package_name:
            raise ValueError("package_name must not be empty")
        if not version:
            raise ValueError("version must not be empty")

        self._package_info = PackageInfo(name=package_name, version=version)

    @property
    def package_info(self) -> PackageInfo:
        """Return package metadata for this client."""
        return self._package_info

    def version(self) -> str:
        """Return the SDK version string."""
        return self._package_info.version

    def health_check(self) -> HealthReport:
        """Run a local SDK health check."""
        report = HealthReport(
            component="python-sdk",
            status=HealthStatus.HEALTHY,
            detail="Python SDK is available",
        )

        if not report.is_healthy:
            raise HealthCheckError(report)

        return report
