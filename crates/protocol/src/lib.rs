//! Protocol types and validation boundaries for codex-sk.
//!
//! This crate should remain transport-neutral. Keep wire formats, schema
//! evolution, and input validation here instead of in runtime or CLI code.

use std::{error::Error, fmt};

/// Supported protocol version for this repository skeleton.
pub const PROTOCOL_VERSION: &str = "0.1.0";

/// Project-wide result type.
pub type ProjectResult<T> = Result<T, ProjectError>;

/// Shared error type for public Rust APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectError {
    /// The caller provided an unsupported or incomplete command.
    InvalidCommand(String),
    /// The caller provided invalid configuration.
    InvalidConfiguration(String),
    /// An I/O operation failed.
    Io(String),
    /// A validation check failed.
    ValidationFailed(String),
    /// A component failed a health check.
    Unhealthy(String),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCommand(command) => write!(formatter, "invalid command: {command}"),
            Self::InvalidConfiguration(message) => {
                write!(formatter, "invalid configuration: {message}")
            }
            Self::Io(message) => write!(formatter, "I/O error: {message}"),
            Self::Unhealthy(component) => write!(formatter, "component is unhealthy: {component}"),
            Self::ValidationFailed(report) => formatter.write_str(report),
        }
    }
}

impl Error for ProjectError {}

/// Health status shared across crates and language bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// The component is ready for local use.
    Healthy,
    /// The component is not ready for local use.
    Unhealthy,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Healthy => formatter.write_str("healthy"),
            Self::Unhealthy => formatter.write_str("unhealthy"),
        }
    }
}

/// Health report for a single component or aggregate check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReport {
    /// Component name.
    pub component: String,
    /// Component status.
    pub status: HealthStatus,
    /// Human-readable detail suitable for CLI output and logs.
    pub detail: String,
}

impl HealthReport {
    /// Creates a healthy report.
    #[must_use]
    pub fn healthy(component: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Healthy,
            detail: detail.into(),
        }
    }

    /// Creates an unhealthy report.
    #[must_use]
    pub fn unhealthy(component: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Unhealthy,
            detail: detail.into(),
        }
    }

    /// Returns true when the report is healthy.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.status == HealthStatus::Healthy
    }
}

/// Returns the supported protocol version.
#[must_use]
pub fn protocol_version() -> &'static str {
    PROTOCOL_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_protocol_version() {
        assert_eq!(protocol_version(), "0.1.0");
    }

    #[test]
    fn formats_health_status() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
    }

    #[test]
    fn creates_health_report() {
        let report = HealthReport::healthy("core", "ready");

        assert_eq!(report.component, "core");
        assert!(report.is_healthy());
    }

    #[test]
    fn formats_project_error() {
        let error = ProjectError::InvalidCommand("serve".to_string());

        assert_eq!(error.to_string(), "invalid command: serve");
    }
}
