//! Command-line entry point for codex-sk.

use std::{env, process::ExitCode};

use codex_sk_core::core_info;
use codex_sk_protocol::{protocol_version, HealthReport, ProjectError, ProjectResult};
use codex_sk_runtime::{check_health, runtime_info};

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(args: impl IntoIterator<Item = String>) -> ProjectResult<String> {
    let args = args.into_iter().collect::<Vec<_>>();

    match args.as_slice() {
        [] => Ok(version_output()),
        [command] if command == "--version" || command == "-V" => Ok(version_output()),
        [command] if command == "check" => health_output(),
        [command, subcommand] if command == "health" && subcommand == "check" => health_output(),
        [command] if command == "--help" || command == "-h" => Ok(help_output()),
        _ => Err(ProjectError::InvalidCommand(args.join(" "))),
    }
}

fn version_output() -> String {
    let info = runtime_info();

    format!(
        "codex-sk {} (protocol {})",
        env!("CARGO_PKG_VERSION"),
        info.protocol_version
    )
}

fn health_output() -> ProjectResult<String> {
    let report = check_health()?;

    Ok(format_health_report(&report))
}

fn format_health_report(report: &HealthReport) -> String {
    let core = core_info();

    format!(
        "{}: {} ({})\ncore: {} {}",
        report.component,
        report.status,
        report.detail,
        core.name,
        protocol_version()
    )
}

fn help_output() -> String {
    [
        "codex-sk",
        "",
        "Usage:",
        "  codex-sk --version",
        "  codex-sk health check",
        "  codex-sk check",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn version_command_reports_versions() {
        let output = run(strings(&["--version"])).expect("version should succeed");

        assert!(output.contains("codex-sk"));
        assert!(output.contains("protocol 0.1.0"));
    }

    #[test]
    fn health_check_command_reports_healthy_runtime() {
        let output = run(strings(&["health", "check"])).expect("health check should succeed");

        assert!(output.contains("runtime: healthy"));
        assert!(output.contains("core: codex-sk-core 0.1.0"));
    }

    #[test]
    fn check_alias_reports_healthy_runtime() {
        let output = run(strings(&["check"])).expect("check alias should succeed");

        assert!(output.contains("runtime: healthy"));
    }

    #[test]
    fn unsupported_command_returns_error() {
        let error = run(strings(&["serve"])).expect_err("serve is not supported");

        assert_eq!(error, ProjectError::InvalidCommand("serve".to_string()));
    }
}
