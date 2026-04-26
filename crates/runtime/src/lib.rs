//! Runtime orchestration for codex-sk.
//!
//! The runtime coordinates core engine behavior with protocol-level inputs.
//! It should not own CLI parsing, Python bindings, or protocol definitions.

use codex_sk_core::{core_info, health_check};
use codex_sk_protocol::{
    protocol_version, HealthReport, HealthStatus, ProjectError, ProjectResult,
};

/// Runtime metadata useful for diagnostics and smoke tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInfo {
    /// Core crate package name.
    pub core_name: &'static str,
    /// Supported protocol version.
    pub protocol_version: &'static str,
}

/// Returns runtime metadata without starting any services.
#[must_use]
pub fn runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        core_name: core_info().name,
        protocol_version: protocol_version(),
    }
}

/// Runs runtime health checks and returns an aggregate report.
///
/// # Errors
///
/// Returns [`ProjectError::Unhealthy`] when a required runtime component fails
/// its health check.
pub fn check_health() -> ProjectResult<HealthReport> {
    let core_report = health_check();

    if core_report.status == HealthStatus::Healthy {
        Ok(HealthReport::healthy(
            "runtime",
            format!(
                "runtime is available; {}: {}",
                core_report.component, core_report.status
            ),
        ))
    } else {
        Err(ProjectError::Unhealthy(core_report.component))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_runtime_metadata() {
        let info = runtime_info();

        assert_eq!(info.core_name, "codex-sk-core");
        assert_eq!(info.protocol_version, "0.1.0");
    }

    #[test]
    fn runtime_health_check_is_healthy() {
        let report = check_health().expect("runtime health check should pass");

        assert_eq!(report.component, "runtime");
        assert!(report.is_healthy());
    }
}
