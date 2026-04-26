"""Typed models exposed by the codex-sk Python SDK."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class HealthStatus(str, Enum):
    """Health status values shared by SDK health checks."""

    HEALTHY = "healthy"
    UNHEALTHY = "unhealthy"


@dataclass(frozen=True)
class PackageInfo:
    """Package metadata exposed by the SDK."""

    name: str
    version: str


@dataclass(frozen=True)
class HealthReport:
    """Health report for a local SDK component."""

    component: str
    status: HealthStatus
    detail: str

    @property
    def is_healthy(self) -> bool:
        """Return true when this report is healthy."""
        return self.status is HealthStatus.HEALTHY
