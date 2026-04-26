import pytest

from codex_sk import CodexSkClient, HealthStatus, package_info
from codex_sk.exceptions import HealthCheckError
from codex_sk.models import HealthReport


def test_package_info() -> None:
    assert package_info().name == "codex-sk"


def test_client_version() -> None:
    client = CodexSkClient()

    assert client.version() == "0.1.0"


def test_client_health_check() -> None:
    client = CodexSkClient()

    report = client.health_check()

    assert report.component == "python-sdk"
    assert report.status is HealthStatus.HEALTHY
    assert report.is_healthy


def test_client_rejects_empty_package_name() -> None:
    with pytest.raises(ValueError, match="package_name"):
        CodexSkClient(package_name="")


def test_health_check_error_includes_report() -> None:
    report = HealthReport(
        component="python-sdk",
        status=HealthStatus.UNHEALTHY,
        detail="not ready",
    )

    error = HealthCheckError(report)

    assert error.report == report
    assert "python-sdk is unhealthy" in str(error)
