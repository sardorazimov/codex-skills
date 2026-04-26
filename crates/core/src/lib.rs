//! Core engine primitives for codex-sk.
//!
//! This crate owns domain concepts that are independent of transport,
//! runtime orchestration, command-line UX, or language bindings.

use codex_sk_protocol::HealthReport;

/// Static package metadata for the core crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreInfo {
    /// Package name as published by Cargo.
    pub name: &'static str,
    /// Package version.
    pub version: &'static str,
}

/// Returns build-time metadata for the core crate.
#[must_use]
pub fn core_info() -> CoreInfo {
    CoreInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    }
}

/// Runs the core crate health check.
#[must_use]
pub fn health_check() -> HealthReport {
    HealthReport::healthy("core", "core crate is available")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_core_metadata() {
        let info = core_info();

        assert_eq!(info.name, "codex-sk-core");
        assert!(!info.version.is_empty());
    }

    #[test]
    fn core_health_check_is_healthy() {
        let report = health_check();

        assert_eq!(report.component, "core");
        assert!(report.is_healthy());
    }
}
