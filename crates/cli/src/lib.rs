//! Command-line implementation for codex-skils.

use std::{env, fs, path::Path, process::ExitCode};

use codex_sk_core::core_info;
use codex_sk_protocol::{protocol_version, HealthReport, ProjectError, ProjectResult};
use codex_sk_runtime::{check_health, runtime_info, start_forwarding_server, ForwarderConfig};

const CONFIG_DIR: &str = ".codex-skils";
const SKILLS_DIR: &str = ".codex-skils/skills";
const CONFIG_PATH: &str = ".codex-skils/config.toml";

const REQUIRED_PATHS: &[RequiredPath] = &[
    RequiredPath::file("README.md"),
    RequiredPath::file("AGENTS.md"),
    RequiredPath::file("CONTRIBUTING.md"),
    RequiredPath::file("SECURITY.md"),
    RequiredPath::file(CONFIG_PATH),
    RequiredPath::dir(SKILLS_DIR),
];

const TEMPLATES: &[SkillTemplate] = &[
    SkillTemplate::new("rust", include_str!("../../../templates/rust.md")),
    SkillTemplate::new("python", include_str!("../../../templates/python.md")),
    SkillTemplate::new(
        "opensource",
        include_str!("../../../templates/opensource.md"),
    ),
    SkillTemplate::new("devops", include_str!("../../../templates/devops.md")),
    SkillTemplate::new("security", include_str!("../../../templates/security.md")),
    SkillTemplate::new("testing", include_str!("../../../templates/testing.md")),
];

/// Runs the command-line entry point.
#[must_use]
pub fn main_entry() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn run(args: impl IntoIterator<Item = String>) -> ProjectResult<String> {
    let args = args.into_iter().collect::<Vec<_>>();
    let root = env::current_dir().map_err(project_io_error)?;

    run_in_root(&root, &args)
}

fn run_in_root(root: &Path, args: &[String]) -> ProjectResult<String> {
    match args {
        [] => Ok(version_output()),
        [command] if command == "--version" || command == "-V" => Ok(version_output()),
        [command] if command == "--help" || command == "-h" => Ok(help_output()),
        [command] if command == "init" => init_project(root, false),
        [command, flag] if command == "init" && flag == "--force" => init_project(root, true),
        [command] if command == "check" => check_project(root),
        [command, subcommand] if command == "health" && subcommand == "check" => health_output(),
        [command, skill_name] if command == "skill" => skill_template_output(skill_name),
        [command, skill_name, flag] if command == "skill" && flag == "--write" => {
            write_skill_template(root, skill_name, false)
        }
        [command, skill_name, write_flag, force_flag]
            if command == "skill" && write_flag == "--write" && force_flag == "--force" =>
        {
            write_skill_template(root, skill_name, true)
        }
        [command, skill_name, force_flag, write_flag]
            if command == "skill" && force_flag == "--force" && write_flag == "--write" =>
        {
            write_skill_template(root, skill_name, true)
        }
        [command, listen_flag, listen_port, target_flag, target_port]
            if command == "start-server"
                && listen_flag == "--listen-port"
                && target_flag == "--target-port" =>
        {
            start_server(listen_port, target_port)
        }
        _ => Err(ProjectError::InvalidCommand(command_text(args))),
    }
}

fn version_output() -> String {
    let info = runtime_info();

    format!(
        "codex-skils {} (protocol {})",
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
        "valid runtime: {} ({})\nvalid core: {} {}\nvalid protocol: {}",
        report.status,
        report.detail,
        core.name,
        core.version,
        protocol_version()
    )
}

fn help_output() -> String {
    let skills = available_skill_names().join(", ");

    format!(
        "{}\n\n{}\n\n{}",
        "codex-skils",
        [
            "Usage:",
            "  codex-skils --version",
            "  codex-skils init [--force]",
            "  codex-skils skill <name>",
            "  codex-skils skill <name> --write [--force]",
            "  codex-skils check",
            "  codex-skils health check",
            "  codex-skils start-server --listen-port <PORT> --target-port <PORT>",
        ]
        .join("\n"),
        format_args!("Available skills: {skills}")
    )
}

fn start_server(listen_port: &str, target_port: &str) -> ProjectResult<String> {
    let config = ForwarderConfig::local_ports(
        parse_port("listen", listen_port)?,
        parse_port("target", target_port)?,
    )?;

    println!(
        "valid configuration: forwarding HTTP requests from {} to {}",
        config.listen_addr, config.target_addr
    );

    start_forwarding_server(config)?;

    Ok("valid server stopped".to_string())
}

fn parse_port(name: &str, value: &str) -> ProjectResult<u16> {
    value.parse().map_err(|_| {
        ProjectError::InvalidConfiguration(format!("{name} port must be a number from 1 to 65535"))
    })
}

fn init_project(root: &Path, force: bool) -> ProjectResult<String> {
    let project_name = project_name(root)?;
    let actions = [
        ensure_dir(root, CONFIG_DIR)?,
        ensure_dir(root, SKILLS_DIR)?,
        write_file(root, "AGENTS.md", agents_template(&project_name), force)?,
        write_file(root, CONFIG_PATH, config_template(&project_name), force)?,
    ];

    Ok(format_actions("init complete", &actions))
}

fn check_project(root: &Path) -> ProjectResult<String> {
    let results = REQUIRED_PATHS
        .iter()
        .map(|required| required.validate(root))
        .collect::<Vec<_>>();

    let report = format_check_report(&results);

    if results.iter().all(CheckItem::is_valid) {
        Ok(report)
    } else {
        Err(ProjectError::ValidationFailed(report))
    }
}

fn skill_template_output(skill_name: &str) -> ProjectResult<String> {
    Ok(template_by_name(skill_name)?.body.to_string())
}

fn write_skill_template(root: &Path, skill_name: &str, force: bool) -> ProjectResult<String> {
    let template = template_by_name(skill_name)?;
    let path = format!("{SKILLS_DIR}/{skill_name}.md");
    let action = write_file(root, &path, template.body.to_string(), force)?;

    Ok(format_actions("skill write complete", &[action]))
}

fn template_by_name(name: &str) -> ProjectResult<&'static SkillTemplate> {
    TEMPLATES
        .iter()
        .find(|template| template.name == name)
        .ok_or_else(|| {
            ProjectError::InvalidCommand(format!(
                "unknown skill '{name}'. Available skills: {}",
                available_skill_names().join(", ")
            ))
        })
}

fn available_skill_names() -> Vec<&'static str> {
    TEMPLATES.iter().map(|template| template.name).collect()
}

fn ensure_dir(root: &Path, relative: &str) -> ProjectResult<FileAction> {
    let path = root.join(relative);

    if path.is_dir() {
        return Ok(FileAction::skipped(relative, "directory already exists"));
    }

    if path.exists() {
        return Err(ProjectError::InvalidConfiguration(format!(
            "{relative} exists but is not a directory"
        )));
    }

    fs::create_dir_all(&path).map_err(project_io_error)?;

    Ok(FileAction::created(relative, "directory created"))
}

fn write_file(
    root: &Path,
    relative: &str,
    contents: String,
    force: bool,
) -> ProjectResult<FileAction> {
    let path = root.join(relative);

    if path.exists() && !force {
        return Ok(FileAction::skipped(relative, "file already exists"));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(project_io_error)?;
    }

    fs::write(&path, contents).map_err(project_io_error)?;

    if force {
        Ok(FileAction::created(relative, "file written with --force"))
    } else {
        Ok(FileAction::created(relative, "file created"))
    }
}

fn agents_template(project_name: &str) -> String {
    format!(
        r"# AGENTS.md

## Project

Project name: {project_name}

## Engineering Rules

- Keep changes small, reviewed, and tested.
- Prefer explicit code over clever abstractions.
- Document public APIs and contributor-facing behavior.
- Validate external input before using it.
- Do not commit secrets or machine-specific configuration.

## Skill Directory

Project skills live in `.codex-skils/skills/`.

## Checks

- Run Rust formatting, clippy, and tests for Rust changes.
- Run Python checks and tests for Python changes.
- Update documentation when behavior or workflow changes.
"
    )
}

fn config_template(project_name: &str) -> String {
    format!(
        r#"schema_version = 1
project_name = "{project_name}"
default_skills = ["rust", "python", "opensource"]
"#
    )
}

fn project_name(root: &Path) -> ProjectResult<String> {
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            ProjectError::InvalidConfiguration(
                "could not determine project directory name".to_string(),
            )
        })?;

    Ok(escape_toml_string(name))
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn format_actions(title: &str, actions: &[FileAction]) -> String {
    let mut lines = vec![title.to_string()];
    lines.extend(actions.iter().map(FileAction::line));
    lines.join("\n")
}

fn format_check_report(items: &[CheckItem]) -> String {
    let mut lines = Vec::with_capacity(items.len() + 1);
    let failed = items.iter().filter(|item| !item.is_valid()).count();

    if failed == 0 {
        lines.push("check passed".to_string());
    } else {
        lines.push(format!("check failed: {failed} required path(s) missing"));
    }

    lines.extend(items.iter().map(CheckItem::line));
    lines.join("\n")
}

fn project_io_error(error: impl std::fmt::Display) -> ProjectError {
    ProjectError::Io(error.to_string())
}

fn command_text(args: &[String]) -> String {
    if args.is_empty() {
        "<empty>".to_string()
    } else {
        args.join(" ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SkillTemplate {
    name: &'static str,
    body: &'static str,
}

impl SkillTemplate {
    const fn new(name: &'static str, body: &'static str) -> Self {
        Self { name, body }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileAction {
    status: FileStatus,
    path: String,
    detail: &'static str,
}

impl FileAction {
    fn created(path: impl Into<String>, detail: &'static str) -> Self {
        Self {
            status: FileStatus::Created,
            path: path.into(),
            detail,
        }
    }

    fn skipped(path: impl Into<String>, detail: &'static str) -> Self {
        Self {
            status: FileStatus::Skipped,
            path: path.into(),
            detail,
        }
    }

    fn line(&self) -> String {
        format!("{} {} ({})", self.status.as_str(), self.path, self.detail)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileStatus {
    Created,
    Skipped,
}

impl FileStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RequiredPath {
    path: &'static str,
    kind: RequiredKind,
}

impl RequiredPath {
    const fn file(path: &'static str) -> Self {
        Self {
            path,
            kind: RequiredKind::File,
        }
    }

    const fn dir(path: &'static str) -> Self {
        Self {
            path,
            kind: RequiredKind::Directory,
        }
    }

    fn validate(self, root: &Path) -> CheckItem {
        let full_path = root.join(self.path);
        let valid = match self.kind {
            RequiredKind::File => full_path.is_file(),
            RequiredKind::Directory => full_path.is_dir(),
        };

        CheckItem {
            path: self.path,
            valid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequiredKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CheckItem {
    path: &'static str,
    valid: bool,
}

impl CheckItem {
    const fn is_valid(&self) -> bool {
        self.valid
    }

    fn line(&self) -> String {
        if self.valid {
            format!("valid {}", self.path)
        } else {
            format!("missing {}", self.path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn version_command_reports_versions() -> ProjectResult<()> {
        let output = run_in_root(&test_root("version")?, &strings(&["--version"]))?;

        assert!(output.contains("codex-skils"));
        assert!(output.contains("protocol 0.1.0"));
        Ok(())
    }

    #[test]
    fn health_check_command_reports_healthy_runtime() -> ProjectResult<()> {
        let output = run_in_root(&test_root("health")?, &strings(&["health", "check"]))?;

        assert!(output.contains("valid runtime: healthy"));
        assert!(output.contains("valid core: codex-sk-core 0.1.0"));
        Ok(())
    }

    #[test]
    fn unsupported_command_returns_error() -> ProjectResult<()> {
        let error = run_in_root(&test_root("unsupported")?, &strings(&["serve"]))
            .err()
            .ok_or_else(|| {
                ProjectError::InvalidConfiguration("expected command error".to_string())
            })?;

        assert_eq!(error, ProjectError::InvalidCommand("serve".to_string()));
        Ok(())
    }

    #[test]
    fn help_includes_product_commands() -> ProjectResult<()> {
        let output = run_in_root(&test_root("help")?, &strings(&["--help"]))?;

        assert!(output.contains("codex-skils init [--force]"));
        assert!(output.contains("codex-skils skill <name> --write [--force]"));
        assert!(output
            .contains("Available skills: rust, python, opensource, devops, security, testing"));
        Ok(())
    }

    #[test]
    fn start_server_rejects_invalid_port() -> ProjectResult<()> {
        let error = run_in_root(
            &test_root("port")?,
            &strings(&[
                "start-server",
                "--listen-port",
                "nope",
                "--target-port",
                "8080",
            ]),
        )
        .err()
        .ok_or_else(|| ProjectError::InvalidConfiguration("expected port error".to_string()))?;

        assert_eq!(
            error,
            ProjectError::InvalidConfiguration(
                "listen port must be a number from 1 to 65535".to_string()
            )
        );
        Ok(())
    }

    #[test]
    fn template_lookup_supports_all_skills() -> ProjectResult<()> {
        for name in [
            "rust",
            "python",
            "opensource",
            "devops",
            "security",
            "testing",
        ] {
            let template = template_by_name(name)?;
            assert!(template.body.starts_with("# "));
            assert!(template.body.contains("##"));
        }

        Ok(())
    }

    #[test]
    fn config_generation_is_valid_simple_toml() {
        let config = config_template("codex-skils");

        assert!(config.contains("schema_version = 1"));
        assert!(config.contains("project_name = \"codex-skils\""));
        assert!(config.contains("default_skills = [\"rust\", \"python\", \"opensource\"]"));
    }

    #[test]
    fn init_creates_required_codex_skils_files() -> ProjectResult<()> {
        let root = test_root("init")?;

        let output = init_project(&root, false)?;

        assert!(output.contains("created .codex-skils"));
        assert!(output.contains("created .codex-skils/skills"));
        assert!(output.contains("created AGENTS.md"));
        assert!(output.contains("created .codex-skils/config.toml"));
        assert!(root.join("AGENTS.md").is_file());
        assert!(root.join(CONFIG_PATH).is_file());
        assert!(root.join(SKILLS_DIR).is_dir());

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn init_skips_existing_files_without_force() -> ProjectResult<()> {
        let root = test_root("init-skip")?;
        write_test_file(&root.join("AGENTS.md"), "custom")?;

        let output = init_project(&root, false)?;
        let agents = fs::read_to_string(root.join("AGENTS.md")).map_err(project_io_error)?;

        assert!(output.contains("skipped AGENTS.md"));
        assert_eq!(agents, "custom");

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn init_force_overwrites_existing_files() -> ProjectResult<()> {
        let root = test_root("init-force")?;
        write_test_file(&root.join("AGENTS.md"), "custom")?;

        let output = init_project(&root, true)?;
        let agents = fs::read_to_string(root.join("AGENTS.md")).map_err(project_io_error)?;

        assert!(output.contains("created AGENTS.md (file written with --force)"));
        assert!(agents.contains("## Engineering Rules"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn check_project_passes_when_required_paths_exist() -> ProjectResult<()> {
        let root = test_root("check-pass")?;
        create_required_paths(&root)?;

        let output = check_project(&root)?;

        assert!(output.starts_with("check passed"));
        assert!(output.contains("valid .codex-skils/config.toml"));
        assert!(output.contains("valid .codex-skils/skills"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn check_project_reports_missing_paths() -> ProjectResult<()> {
        let root = test_root("check-fail")?;
        write_test_file(&root.join("README.md"), "")?;

        let error = check_project(&root).err().ok_or_else(|| {
            ProjectError::InvalidConfiguration("expected check failure".to_string())
        })?;

        let message = error.to_string();
        assert!(message.contains("check failed"));
        assert!(message.contains("missing AGENTS.md"));
        assert!(message.contains("missing .codex-skils/config.toml"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn skill_command_prints_template_by_default() -> ProjectResult<()> {
        let root = test_root("skill-print")?;

        let output = run_in_root(&root, &strings(&["skill", "devops"]))?;

        assert!(output.contains("# DevOps Skill"));
        assert!(!root.join(".codex-skils/skills/devops.md").exists());

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn write_skill_template_creates_skill_file() -> ProjectResult<()> {
        let root = test_root("skill-write")?;

        let output = write_skill_template(&root, "python", false)?;
        let path = root.join(".codex-skils/skills/python.md");
        let contents = fs::read_to_string(path).map_err(project_io_error)?;

        assert!(output.contains("created .codex-skils/skills/python.md"));
        assert!(contents.contains("# Python SDK Skill"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn write_skill_template_does_not_overwrite_without_force() -> ProjectResult<()> {
        let root = test_root("skill-no-overwrite")?;
        let path = root.join(".codex-skils/skills/security.md");
        write_test_file(&path, "custom")?;

        let output = write_skill_template(&root, "security", false)?;
        let contents = fs::read_to_string(path).map_err(project_io_error)?;

        assert!(output.contains("skipped .codex-skils/skills/security.md"));
        assert_eq!(contents, "custom");

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn write_skill_template_overwrites_with_force() -> ProjectResult<()> {
        let root = test_root("skill-force")?;
        let path = root.join(".codex-skils/skills/testing.md");
        write_test_file(&path, "custom")?;

        let output = write_skill_template(&root, "testing", true)?;
        let contents = fs::read_to_string(path).map_err(project_io_error)?;

        assert!(output.contains("created .codex-skils/skills/testing.md"));
        assert!(contents.contains("# Testing Skill"));

        cleanup(&root)?;
        Ok(())
    }

    fn create_required_paths(root: &Path) -> ProjectResult<()> {
        for path in ["README.md", "AGENTS.md", "CONTRIBUTING.md", "SECURITY.md"] {
            write_test_file(&root.join(path), "")?;
        }
        write_test_file(&root.join(CONFIG_PATH), "schema_version = 1\n")?;
        fs::create_dir_all(root.join(SKILLS_DIR)).map_err(project_io_error)
    }

    fn test_root(name: &str) -> ProjectResult<PathBuf> {
        let root = env::temp_dir().join(format!("codex-skils-{name}-{}", std::process::id()));
        cleanup(&root)?;
        fs::create_dir_all(&root).map_err(project_io_error)?;
        Ok(root)
    }

    fn write_test_file(path: &Path, contents: &str) -> ProjectResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(project_io_error)?;
        }
        fs::write(path, contents).map_err(project_io_error)
    }

    fn cleanup(root: &Path) -> ProjectResult<()> {
        if root.exists() {
            fs::remove_dir_all(root).map_err(project_io_error)?;
        }
        Ok(())
    }
}
