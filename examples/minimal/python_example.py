"""Minimal Python SDK example for codex-sk."""

from codex_sk import CodexSkClient


def main() -> None:
    """Run a local SDK health check."""
    client = CodexSkClient()
    report = client.health_check()

    print(f"codex-sk Python SDK {client.version()}")
    print(f"{report.component}: {report.status.value} ({report.detail})")


if __name__ == "__main__":
    main()
