"""Exceptions raised by the codex-sk Python SDK."""

from __future__ import annotations

from codex_sk.models import HealthReport


class CodexSkError(Exception):
    """Base exception for codex-sk SDK errors."""


class HealthCheckError(CodexSkError):
    """Raised when a health check reports an unhealthy component."""

    def __init__(self, report: HealthReport) -> None:
        """Create an error from a failed health report."""
        self.report = report
        super().__init__(f"{report.component} is {report.status.value}: {report.detail}")
