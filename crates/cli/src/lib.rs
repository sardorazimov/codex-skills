//! Command-line implementation for codex-skils.

use std::{env, fs, path::Path, process::ExitCode};

use codex_sk_core::core_info;
use codex_sk_protocol::{protocol_version, HealthReport, ProjectError, ProjectResult};
use codex_sk_runtime::{check_health, runtime_info, start_forwarding_server, ForwarderConfig};

const CONFIG_DIR: &str = ".codex-skils";
const SKILLS_DIR: &str = ".codex-skils/skills";
const CONFIG_PATH: &str = ".codex-skils/config.toml";
const SKILLS_BEGIN_MARKER: &str = "<!-- codex-skils:start -->";
const SKILLS_END_MARKER: &str = "<!-- codex-skils:end -->";
const README_RULES_BEGIN_MARKER: &str = "<!-- codex-skils:readme:start -->";
const README_RULES_END_MARKER: &str = "<!-- codex-skils:readme:end -->";

const REQUIRED_PATHS: &[RequiredPath] = &[
    RequiredPath::file("README.md"),
    RequiredPath::file("AGENTS.md"),
    RequiredPath::file("CONTRIBUTING.md"),
    RequiredPath::file("SECURITY.md"),
    RequiredPath::file(CONFIG_PATH),
    RequiredPath::dir(SKILLS_DIR),
];

const TEMPLATES: &[SkillTemplate] = &[
    SkillTemplate::new(
        "rust",
        "Rust crate, CLI, runtime, and protocol engineering.",
        include_str!("../../../templates/rust.md"),
    ),
    SkillTemplate::new(
        "python",
        "Python SDK and developer-facing API work.",
        include_str!("../../../templates/python.md"),
    ),
    SkillTemplate::new(
        "opensource",
        "Contributor workflow and maintainer documentation.",
        include_str!("../../../templates/opensource.md"),
    ),
    SkillTemplate::new(
        "devops",
        "CI, release checks, scripts, and automation.",
        include_str!("../../../templates/devops.md"),
    ),
    SkillTemplate::new(
        "security",
        "Validation, secret handling, and security-sensitive changes.",
        include_str!("../../../templates/security.md"),
    ),
    SkillTemplate::new(
        "testing",
        "Test strategy, fixtures, and regression coverage.",
        include_str!("../../../templates/testing.md"),
    ),
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
        [command, flags @ ..] if command == "apply" => {
            apply_skills(root, ApplyOptions::parse(flags)?)
        }
        [command] if command == "codex" => generate_codex_prompt(root, None),
        [command, flag] if command == "codex" && flag == "--print" => {
            generate_codex_prompt(root, None)
        }
        [command, output_flag, output] if command == "codex" && output_flag == "--output" => {
            generate_codex_prompt(root, Some(output))
        }
        [command] if command == "list" => Ok(list_skills()),
        [command] if command == "check" => check_project(root),
        [command, subcommand] if command == "health" && subcommand == "check" => health_output(),
        [command, skill_name] if command == "skill" => {
            skill_template_output(skill_name, OutputFormat::Markdown)
        }
        [command, skill_name, format_flag, format]
            if command == "skill" && format_flag == "--format" =>
        {
            skill_template_output(skill_name, OutputFormat::parse(format)?)
        }
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
        [command, all_flag] if command == "export" && all_flag == "--all" => {
            export_all(root, OutputFormat::Markdown, None)
        }
        [command, all_flag, format_flag, format]
            if command == "export" && all_flag == "--all" && format_flag == "--format" =>
        {
            export_all(root, OutputFormat::parse(format)?, None)
        }
        [command, all_flag, output_flag, output]
            if command == "export" && all_flag == "--all" && output_flag == "--output" =>
        {
            export_all(root, OutputFormat::Markdown, Some(output))
        }
        [command, all_flag, format_flag, format, output_flag, output]
            if command == "export"
                && all_flag == "--all"
                && format_flag == "--format"
                && output_flag == "--output" =>
        {
            export_all(root, OutputFormat::parse(format)?, Some(output))
        }
        [command, all_flag, output_flag, output, format_flag, format]
            if command == "export"
                && all_flag == "--all"
                && output_flag == "--output"
                && format_flag == "--format" =>
        {
            export_all(root, OutputFormat::parse(format)?, Some(output))
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
            "  codex-skils apply [--force] [--dry-run] [--readme]",
            "  codex-skils codex [--print|--output <FILE>]",
            "  codex-skils list",
            "  codex-skils skill <name> [--format markdown|json|yaml]",
            "  codex-skils skill <name> --write [--force]",
            "  codex-skils export --all [--format markdown|json|yaml] [--output <DIR>]",
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

fn generate_codex_prompt(root: &Path, output: Option<&str>) -> ProjectResult<String> {
    let agents = read_optional_file(&root.join("AGENTS.md"))?;
    let skills = read_project_skills_optional(root)?;
    let prompt = build_codex_prompt(agents.as_deref(), &skills);
    let summary = format!("codex prompt generated\nmerged {} skill(s)", skills.len());

    if let Some(output) = output {
        let path = root.join(output);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(project_io_error)?;
        }
        fs::write(path, prompt).map_err(project_io_error)?;
        Ok(format!("{summary}\nwrote {output}"))
    } else {
        Ok(format!("{summary}\n\n{prompt}"))
    }
}

fn read_optional_file(path: &Path) -> ProjectResult<Option<String>> {
    if path.is_file() {
        fs::read_to_string(path).map(Some).map_err(project_io_error)
    } else {
        Ok(None)
    }
}

fn read_project_skills_optional(root: &Path) -> ProjectResult<Vec<ProjectSkill>> {
    let skills_dir = root.join(SKILLS_DIR);
    if !skills_dir.is_dir() {
        return Ok(Vec::new());
    }

    read_project_skills_from_dir(&skills_dir, false)
}

fn build_codex_prompt(agents: Option<&str>, skills: &[ProjectSkill]) -> String {
    let mut sections = Vec::new();

    if let Some(agents) = agents.and_then(non_empty_trimmed) {
        if let Some(cleaned) = non_empty_trimmed(&strip_codex_managed_sections(agents)) {
            sections.push(cleaned.to_string());
        }
    }

    sections.extend(deduplicated_skill_sections(skills));

    let rules = if sections.is_empty() {
        "No repository-specific skills were found.".to_string()
    } else {
        sections.join("\n\n")
    };

    format!(
        "You are a senior software engineer working in this repository.\n\nFollow these rules strictly:\n\n{rules}\n\nOperating instructions:\n- Make minimal changes\n- Keep code consistent\n- Run required checks before finishing\n- Do not break existing behavior\n- Explain changes briefly"
    )
}

fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn strip_codex_managed_sections(content: &str) -> String {
    let without_skills = remove_complete_section(content, SKILLS_BEGIN_MARKER, SKILLS_END_MARKER);
    remove_complete_section(
        &without_skills,
        README_RULES_BEGIN_MARKER,
        README_RULES_END_MARKER,
    )
}

fn remove_complete_section(content: &str, begin_marker: &str, end_marker: &str) -> String {
    let mut remaining = content.to_string();

    while let Ok(Some((start, end))) = find_managed_section(&remaining, begin_marker, end_marker) {
        remaining.replace_range(start..end, "");
    }

    remaining.trim().to_string()
}

fn deduplicated_skill_sections(skills: &[ProjectSkill]) -> Vec<String> {
    let mut sections = Vec::new();
    let mut seen_headings = Vec::new();

    for skill in skills {
        for section in split_markdown_sections(&skill.content) {
            let key = section_key(&section);
            if seen_headings.iter().any(|seen| seen == &key) {
                continue;
            }
            seen_headings.push(key);
            sections.push(section);
        }
    }

    sections
}

fn split_markdown_sections(content: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut current = Vec::new();

    for line in content.trim().lines() {
        if line.starts_with("# ") && !current.is_empty() {
            sections.push(current.join("\n").trim().to_string());
            current.clear();
        }
        current.push(line.to_string());
    }

    if !current.is_empty() {
        sections.push(current.join("\n").trim().to_string());
    }

    sections
        .into_iter()
        .filter(|section| !section.is_empty())
        .collect()
}

fn section_key(section: &str) -> String {
    section
        .lines()
        .next()
        .map_or(section, str::trim)
        .to_ascii_lowercase()
}

fn apply_skills(root: &Path, options: ApplyOptions) -> ProjectResult<String> {
    let skills = read_project_skills(root)?;
    let mut changes = Vec::new();

    changes.push(apply_managed_section(
        root,
        "AGENTS.md",
        "## Skills",
        SKILLS_BEGIN_MARKER,
        SKILLS_END_MARKER,
        &format_skills_section(&skills),
        ApplyTarget {
            force: options.force,
            dry_run: options.dry_run,
            create_if_missing: true,
        },
    )?);

    if options.update_readme {
        changes.push(apply_managed_section(
            root,
            "README.md",
            "## Development Rules",
            README_RULES_BEGIN_MARKER,
            README_RULES_END_MARKER,
            &format_development_rules_section(&skills),
            ApplyTarget {
                force: options.force,
                dry_run: options.dry_run,
                create_if_missing: true,
            },
        )?);
    }

    Ok(format_apply_report(skills.len(), options.dry_run, &changes))
}

fn read_project_skills(root: &Path) -> ProjectResult<Vec<ProjectSkill>> {
    let skills_dir = root.join(SKILLS_DIR);

    if !skills_dir.is_dir() {
        return Err(ProjectError::InvalidConfiguration(format!(
            "no skills found in {SKILLS_DIR}"
        )));
    }

    read_project_skills_from_dir(&skills_dir, true)
}

fn read_project_skills_from_dir(
    skills_dir: &Path,
    require_non_empty: bool,
) -> ProjectResult<Vec<ProjectSkill>> {
    let mut paths = fs::read_dir(skills_dir)
        .map_err(project_io_error)?
        .map(|entry| entry.map(|entry| entry.path()).map_err(project_io_error))
        .collect::<ProjectResult<Vec<_>>>()?;

    paths.sort();

    let mut skills = Vec::new();
    for path in paths {
        if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
            continue;
        }

        let Some(name) = path
            .file_stem()
            .and_then(|name| name.to_str())
            .map(ToString::to_string)
        else {
            continue;
        };

        let content = fs::read_to_string(&path).map_err(project_io_error)?;
        if content.trim().is_empty() {
            continue;
        }

        skills.push(ProjectSkill { name, content });
    }

    if skills.is_empty() && require_non_empty {
        return Err(ProjectError::InvalidConfiguration(format!(
            "no skills found in {SKILLS_DIR}"
        )));
    }

    Ok(skills)
}

fn apply_managed_section(
    root: &Path,
    relative: &str,
    heading: &str,
    begin_marker: &str,
    end_marker: &str,
    body: &str,
    target: ApplyTarget,
) -> ProjectResult<ChangeSummary> {
    let path = root.join(relative);
    if !path.exists() && target.create_if_missing {
        let section = managed_section(heading, begin_marker, end_marker, body);
        if !target.dry_run {
            fs::write(path, section).map_err(project_io_error)?;
        }
        return Ok(ChangeSummary {
            status: target.status_for_write(),
            path: relative.to_string(),
            detail: "file created with managed section",
        });
    }

    if !path.is_file() {
        return Err(ProjectError::InvalidConfiguration(format!(
            "missing required file: {relative}"
        )));
    }

    let original = fs::read_to_string(&path).map_err(project_io_error)?;
    let section = managed_section(heading, begin_marker, end_marker, body);

    let (updated, status, detail) = match find_managed_section(&original, begin_marker, end_marker)
    {
        Ok(Some((start, end))) => {
            let current = &original[start..end];
            if current == section {
                (
                    original.clone(),
                    ChangeStatus::Unchanged,
                    "managed section already up to date",
                )
            } else {
                let mut next = String::new();
                next.push_str(&original[..start]);
                next.push_str(&section);
                next.push_str(&original[end..]);
                (next, target.status_for_write(), "managed section replaced")
            }
        }
        Ok(None) => (
            append_section(&original, &section),
            target.status_for_write(),
            "managed section added",
        ),
        Err(message) if target.force => {
            let cleaned = remove_managed_markers(&original, begin_marker, end_marker);
            (
                append_section(&cleaned, &section),
                target.status_for_write(),
                "malformed managed section rebuilt",
            )
        }
        Err(error) => return Err(error),
    };

    if updated != original && !target.dry_run {
        fs::write(path, updated).map_err(project_io_error)?;
    }

    Ok(ChangeSummary {
        status,
        path: relative.to_string(),
        detail,
    })
}

fn managed_section(heading: &str, begin_marker: &str, end_marker: &str, body: &str) -> String {
    format!("{begin_marker}\n{heading}\n\n{body}\n\n{end_marker}\n")
}

fn append_section(original: &str, section: &str) -> String {
    let mut updated = original.trim_end().to_string();
    if !updated.is_empty() {
        updated.push_str("\n\n");
    }
    updated.push_str(section);
    updated
}

fn find_managed_section(
    content: &str,
    begin_marker: &str,
    end_marker: &str,
) -> ProjectResult<Option<(usize, usize)>> {
    let begin = content.find(begin_marker);
    let end = content.find(end_marker);

    let (Some(begin), Some(end_marker_start)) = (begin, end) else {
        return if begin.is_some() || end.is_some() {
            Err(ProjectError::InvalidConfiguration(format!(
                "malformed managed section markers for {begin_marker} / {end_marker}; use --force to rebuild"
            )))
        } else {
            Ok(None)
        };
    };

    if end_marker_start < begin {
        return Err(ProjectError::InvalidConfiguration(format!(
            "malformed managed section markers for {begin_marker} / {end_marker}; use --force to rebuild"
        )));
    }

    let mut end = end_marker_start + end_marker.len();

    if content[end..].starts_with('\n') {
        end += 1;
    }

    Ok(Some((begin, end)))
}

fn remove_managed_markers(content: &str, begin_marker: &str, end_marker: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed != begin_marker && trimmed != end_marker
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_skills_section(skills: &[ProjectSkill]) -> String {
    skills
        .iter()
        .map(|skill| format!("### {}\n\n{}", skill.name, skill.content.trim()))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn format_development_rules_section(skills: &[ProjectSkill]) -> String {
    let mut lines = vec![
        "This project uses codex-skils to manage AI/Codex engineering rules.".to_string(),
        String::new(),
        "Active skills:".to_string(),
        String::new(),
    ];
    lines.extend(skills.iter().map(|skill| format!("- {}", skill.name)));
    lines.join("\n")
}

fn list_skills() -> String {
    let mut lines = vec!["available skills".to_string()];
    lines.extend(
        TEMPLATES
            .iter()
            .map(|template| format!("{} - {}", template.name, template.description)),
    );
    lines.join("\n")
}

fn skill_template_output(skill_name: &str, format: OutputFormat) -> ProjectResult<String> {
    Ok(format_skill(template_by_name(skill_name)?, format))
}

fn write_skill_template(root: &Path, skill_name: &str, force: bool) -> ProjectResult<String> {
    let template = template_by_name(skill_name)?;
    let path = format!("{SKILLS_DIR}/{skill_name}.md");
    let action = write_file(root, &path, template.body.to_string(), force)?;

    Ok(format_actions("skill write complete", &[action]))
}

fn export_all(root: &Path, format: OutputFormat, output: Option<&str>) -> ProjectResult<String> {
    match output {
        Some(output_dir) => export_all_to_dir(root, format, output_dir),
        None => Ok(format_all_skills(format)),
    }
}

fn export_all_to_dir(root: &Path, format: OutputFormat, output_dir: &str) -> ProjectResult<String> {
    let output_root = root.join(output_dir);
    fs::create_dir_all(&output_root).map_err(project_io_error)?;

    match format {
        OutputFormat::Markdown => {
            let actions = TEMPLATES
                .iter()
                .map(|template| {
                    let relative = format!("{output_dir}/{}.md", template.name);
                    write_file(root, &relative, template.body.to_string(), true)
                })
                .collect::<ProjectResult<Vec<_>>>()?;
            Ok(format_actions("export complete", &actions))
        }
        OutputFormat::Json => {
            let relative = format!("{output_dir}/skills.json");
            let action = write_file(root, &relative, format_all_skills(format), true)?;
            Ok(format_actions("export complete", &[action]))
        }
        OutputFormat::Yaml => {
            let relative = format!("{output_dir}/skills.yaml");
            let action = write_file(root, &relative, format_all_skills(format), true)?;
            Ok(format_actions("export complete", &[action]))
        }
    }
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

fn format_apply_report(skill_count: usize, dry_run: bool, changes: &[ChangeSummary]) -> String {
    let mut lines = vec![
        "apply complete".to_string(),
        format!("found {skill_count} skill(s)"),
    ];
    lines.extend(changes.iter().map(ChangeSummary::line));
    if dry_run {
        lines.push("dry run: no files changed".to_string());
    }
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
    description: &'static str,
    body: &'static str,
}

impl SkillTemplate {
    const fn new(name: &'static str, description: &'static str, body: &'static str) -> Self {
        Self {
            name,
            description,
            body,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectSkill {
    name: String,
    content: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ApplyOptions {
    update_readme: bool,
    force: bool,
    dry_run: bool,
}

impl ApplyOptions {
    fn parse(flags: &[String]) -> ProjectResult<Self> {
        let mut options = Self::default();

        for flag in flags {
            match flag.as_str() {
                "--readme" => options.update_readme = true,
                "--force" => options.force = true,
                "--dry-run" => options.dry_run = true,
                _ => {
                    return Err(ProjectError::InvalidCommand(format!(
                        "apply {}",
                        flags.join(" ")
                    )))
                }
            }
        }

        Ok(options)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ApplyTarget {
    force: bool,
    dry_run: bool,
    create_if_missing: bool,
}

impl ApplyTarget {
    const fn status_for_write(self) -> ChangeStatus {
        if self.dry_run {
            ChangeStatus::WouldUpdate
        } else {
            ChangeStatus::Updated
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Markdown,
    Json,
    Yaml,
}

impl OutputFormat {
    fn parse(value: &str) -> ProjectResult<Self> {
        match value {
            "markdown" => Ok(Self::Markdown),
            "json" => Ok(Self::Json),
            "yaml" => Ok(Self::Yaml),
            _ => Err(ProjectError::InvalidConfiguration(format!(
                "unknown format '{value}'. Supported formats: markdown, json, yaml"
            ))),
        }
    }
}

fn format_skill(template: &SkillTemplate, format: OutputFormat) -> String {
    match format {
        OutputFormat::Markdown => template.body.to_string(),
        OutputFormat::Json => format!(
            "{{\n  \"name\": \"{}\",\n  \"description\": \"{}\",\n  \"content\": \"{}\"\n}}",
            json_escape(template.name),
            json_escape(template.description),
            json_escape(template.body)
        ),
        OutputFormat::Yaml => format!(
            "name: {}\ndescription: {}\ncontent: |\n{}",
            yaml_scalar(template.name),
            yaml_scalar(template.description),
            indent_block(template.body)
        ),
    }
}

fn format_all_skills(format: OutputFormat) -> String {
    match format {
        OutputFormat::Markdown => TEMPLATES
            .iter()
            .map(|template| template.body.trim_end())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n"),
        OutputFormat::Json => {
            let skills = TEMPLATES
                .iter()
                .map(|template| {
                    format!(
                        "    {{\n      \"name\": \"{}\",\n      \"description\": \"{}\",\n      \"content\": \"{}\"\n    }}",
                        json_escape(template.name),
                        json_escape(template.description),
                        json_escape(template.body)
                    )
                })
                .collect::<Vec<_>>()
                .join(",\n");
            format!("{{\n  \"skills\": [\n{skills}\n  ]\n}}")
        }
        OutputFormat::Yaml => {
            let skills = TEMPLATES
                .iter()
                .map(|template| {
                    format!(
                        "  - name: {}\n    description: {}\n    content: |\n{}",
                        yaml_scalar(template.name),
                        yaml_scalar(template.description),
                        indent_block_with(template.body, "      ")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("skills:\n{skills}")
        }
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value if value.is_control() => {
                push_json_unicode_escape(&mut escaped, value);
            }
            value => escaped.push(value),
        }
    }

    escaped
}

fn push_json_unicode_escape(output: &mut String, character: char) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let value = character as u32;

    output.push_str("\\u");
    for shift in [12, 8, 4, 0] {
        let index = ((value >> shift) & 0x0f) as usize;
        output.push(HEX[index] as char);
    }
}

fn yaml_scalar(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn indent_block(value: &str) -> String {
    indent_block_with(value, "  ")
}

fn indent_block_with(value: &str, prefix: &str) -> String {
    value
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileAction {
    status: FileStatus,
    path: String,
    detail: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChangeSummary {
    status: ChangeStatus,
    path: String,
    detail: &'static str,
}

impl ChangeSummary {
    fn line(&self) -> String {
        format!("{} {} ({})", self.status.as_str(), self.path, self.detail)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChangeStatus {
    Updated,
    Unchanged,
    WouldUpdate,
}

impl ChangeStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Updated => "updated",
            Self::Unchanged => "unchanged",
            Self::WouldUpdate => "would update",
        }
    }
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
        assert!(output.contains("codex-skils apply [--force] [--dry-run] [--readme]"));
        assert!(output.contains("codex-skils codex [--print|--output <FILE>]"));
        assert!(output.contains("codex-skils list"));
        assert!(output.contains("codex-skils skill <name> [--format markdown|json|yaml]"));
        assert!(output.contains("codex-skils skill <name> --write [--force]"));
        assert!(output.contains("codex-skils export --all [--format markdown|json|yaml]"));
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
    fn codex_mode_works_with_no_skills() -> ProjectResult<()> {
        let root = test_root("codex-no-skills")?;
        write_test_file(
            &root.join("AGENTS.md"),
            "# AGENTS.md\n\nFollow local rules.",
        )?;

        let output = run_in_root(&root, &strings(&["codex"]))?;

        assert!(output.starts_with("codex prompt generated\nmerged 0 skill(s)"));
        assert!(output.contains("You are a senior software engineer working in this repository."));
        assert!(output.contains("Follow local rules."));
        assert!(output.contains("Operating instructions:"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn codex_mode_merges_multiple_skills() -> ProjectResult<()> {
        let root = test_root("codex-multiple-skills")?;
        write_test_file(&root.join("AGENTS.md"), "# AGENTS.md\n")?;
        write_test_file(
            &root.join(".codex-skils/skills/rust.md"),
            "# Rust\n\nUse Rust.",
        )?;
        write_test_file(
            &root.join(".codex-skils/skills/security.md"),
            "# Security\n\nValidate input.",
        )?;

        let output = run_in_root(&root, &strings(&["codex", "--print"]))?;

        assert!(output.contains("merged 2 skill(s)"));
        assert!(output.contains("# Rust"));
        assert!(output.contains("# Security"));
        assert!(output.contains("- Make minimal changes"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn codex_mode_removes_duplicate_sections() -> ProjectResult<()> {
        let root = test_root("codex-dedupe")?;
        write_test_file(&root.join(".codex-skils/skills/a.md"), "# Shared\n\nFirst.")?;
        write_test_file(
            &root.join(".codex-skils/skills/b.md"),
            "# Shared\n\nSecond.",
        )?;

        let output = run_in_root(&root, &strings(&["codex"]))?;

        assert_eq!(output.matches("# Shared").count(), 1);
        assert!(output.contains("First."));
        assert!(!output.contains("Second."));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn codex_mode_ignores_existing_managed_agents_skills() -> ProjectResult<()> {
        let root = test_root("codex-managed-agents")?;
        let agents = format!(
            "# AGENTS.md\n\n{SKILLS_BEGIN_MARKER}\n## Skills\n\n### rust\n\n# Rust\n\nOld.\n\n{SKILLS_END_MARKER}\n"
        );
        write_test_file(&root.join("AGENTS.md"), &agents)?;
        write_test_file(
            &root.join(".codex-skils/skills/rust.md"),
            "# Rust\n\nCurrent.",
        )?;

        let output = run_in_root(&root, &strings(&["codex"]))?;

        assert_eq!(output.matches("# Rust").count(), 1);
        assert!(!output.contains("Old."));
        assert!(output.contains("Current."));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn codex_mode_writes_output_file() -> ProjectResult<()> {
        let root = test_root("codex-output")?;
        write_test_file(
            &root.join(".codex-skils/skills/testing.md"),
            "# Testing\n\nTest well.",
        )?;

        let output = run_in_root(&root, &strings(&["codex", "--output", "prompt.txt"]))?;
        let prompt = fs::read_to_string(root.join("prompt.txt")).map_err(project_io_error)?;

        assert_eq!(
            output,
            "codex prompt generated\nmerged 1 skill(s)\nwrote prompt.txt"
        );
        assert!(prompt.contains("You are a senior software engineer working in this repository."));
        assert!(prompt.contains("# Testing"));
        assert!(prompt.contains("- Explain changes briefly"));

        cleanup(&root)?;
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
    fn list_prints_all_skills_with_descriptions() -> ProjectResult<()> {
        let root = test_root("list")?;

        let output = run_in_root(&root, &strings(&["list"]))?;

        for name in [
            "rust",
            "python",
            "opensource",
            "devops",
            "security",
            "testing",
        ] {
            assert!(output.contains(name));
        }
        assert!(output.contains("Rust crate, CLI, runtime, and protocol engineering."));
        assert!(output.contains("Test strategy, fixtures, and regression coverage."));

        cleanup(&root)?;
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
    fn skill_markdown_format_prints_template() -> ProjectResult<()> {
        let root = test_root("skill-markdown")?;

        let output = run_in_root(&root, &strings(&["skill", "rust", "--format", "markdown"]))?;

        assert!(output.starts_with("# Rust Engineering Skill"));
        assert!(output.contains("## Required Checks"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn skill_json_format_includes_name_description_and_content() -> ProjectResult<()> {
        let root = test_root("skill-json")?;

        let output = run_in_root(&root, &strings(&["skill", "python", "--format", "json"]))?;

        assert!(output.contains("\"name\": \"python\""));
        assert!(output.contains("\"description\": \"Python SDK and developer-facing API work.\""));
        assert!(output.contains("\"content\": \"# Python SDK Skill\\n"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn skill_yaml_format_includes_name_description_and_content() -> ProjectResult<()> {
        let root = test_root("skill-yaml")?;

        let output = run_in_root(&root, &strings(&["skill", "security", "--format", "yaml"]))?;

        assert!(output.contains("name: \"security\""));
        assert!(output.contains(
            "description: \"Validation, secret handling, and security-sensitive changes.\""
        ));
        assert!(output.contains("content: |\n  # Security Skill"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn unknown_format_returns_useful_error() -> ProjectResult<()> {
        let root = test_root("unknown-format")?;

        let error = run_in_root(&root, &strings(&["skill", "rust", "--format", "toml"]))
            .err()
            .ok_or_else(|| {
                ProjectError::InvalidConfiguration("expected format error".to_string())
            })?;

        assert_eq!(
            error,
            ProjectError::InvalidConfiguration(
                "unknown format 'toml'. Supported formats: markdown, json, yaml".to_string()
            )
        );

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn export_all_prints_markdown_by_default() -> ProjectResult<()> {
        let root = test_root("export-markdown")?;

        let output = run_in_root(&root, &strings(&["export", "--all"]))?;

        assert!(output.contains("# Rust Engineering Skill"));
        assert!(output.contains("# Testing Skill"));
        assert!(output.contains("\n\n---\n\n"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn export_all_prints_json() -> ProjectResult<()> {
        let root = test_root("export-json")?;

        let output = run_in_root(&root, &strings(&["export", "--all", "--format", "json"]))?;

        assert!(output.contains("\"skills\": ["));
        assert!(output.contains("\"name\": \"rust\""));
        assert!(
            output.contains("\"description\": \"CI, release checks, scripts, and automation.\"")
        );

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn export_output_directory_writes_markdown_files() -> ProjectResult<()> {
        let root = test_root("export-output-markdown")?;

        let output = run_in_root(
            &root,
            &strings(&["export", "--all", "--output", ".codex-skils/export"]),
        )?;

        assert!(output.contains("created .codex-skils/export/rust.md"));
        assert!(root.join(".codex-skils/export/rust.md").is_file());
        assert!(root.join(".codex-skils/export/testing.md").is_file());

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn export_output_directory_writes_json_file() -> ProjectResult<()> {
        let root = test_root("export-output-json")?;

        let output = run_in_root(
            &root,
            &strings(&[
                "export",
                "--all",
                "--format",
                "json",
                "--output",
                ".codex-skils/export",
            ]),
        )?;
        let contents = fs::read_to_string(root.join(".codex-skils/export/skills.json"))
            .map_err(project_io_error)?;

        assert!(output.contains("created .codex-skils/export/skills.json"));
        assert!(contents.contains("\"name\": \"security\""));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn export_output_directory_writes_yaml_file() -> ProjectResult<()> {
        let root = test_root("export-output-yaml")?;

        let output = run_in_root(
            &root,
            &strings(&[
                "export",
                "--all",
                "--output",
                ".codex-skils/export",
                "--format",
                "yaml",
            ]),
        )?;
        let contents = fs::read_to_string(root.join(".codex-skils/export/skills.yaml"))
            .map_err(project_io_error)?;

        assert!(output.contains("created .codex-skils/export/skills.yaml"));
        assert!(contents.contains("name: \"testing\""));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn apply_fails_when_no_skills_exist() -> ProjectResult<()> {
        let root = test_root("apply-no-skills")?;
        fs::create_dir_all(root.join(SKILLS_DIR)).map_err(project_io_error)?;

        let error = run_in_root(&root, &strings(&["apply"]))
            .err()
            .ok_or_else(|| {
                ProjectError::InvalidConfiguration("expected apply error".to_string())
            })?;

        assert_eq!(
            error,
            ProjectError::InvalidConfiguration(format!("no skills found in {SKILLS_DIR}"))
        );

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn apply_creates_agents_section() -> ProjectResult<()> {
        let root = test_root("apply-create-agents")?;
        write_test_file(
            &root.join(".codex-skils/skills/rust.md"),
            "# Rust Skill\n\nUse Rust.",
        )?;

        let output = run_in_root(&root, &strings(&["apply"]))?;
        let agents = fs::read_to_string(root.join("AGENTS.md")).map_err(project_io_error)?;

        assert!(output.contains("apply complete"));
        assert!(output.contains("found 1 skill(s)"));
        assert!(output.contains("updated AGENTS.md (file created with managed section)"));
        assert!(agents.starts_with(SKILLS_BEGIN_MARKER));
        assert!(agents.contains("## Skills"));
        assert!(agents.contains("### rust"));
        assert!(agents.contains("# Rust Skill"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn apply_updates_existing_managed_section() -> ProjectResult<()> {
        let root = test_root("apply-update-managed")?;
        let agents = format!("{SKILLS_BEGIN_MARKER}\n## Skills\n\nold\n\n{SKILLS_END_MARKER}\n");
        write_test_file(&root.join("AGENTS.md"), &agents)?;
        write_test_file(
            &root.join(".codex-skils/skills/security.md"),
            "# Security Skill\n",
        )?;

        let output = run_in_root(&root, &strings(&["apply"]))?;
        let agents = fs::read_to_string(root.join("AGENTS.md")).map_err(project_io_error)?;

        assert!(output.contains("updated AGENTS.md (managed section replaced)"));
        assert!(!agents.contains("old"));
        assert!(agents.contains("# Security Skill"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn apply_does_not_duplicate_content() -> ProjectResult<()> {
        let root = test_root("apply-idempotent")?;
        write_test_file(&root.join(".codex-skils/skills/rust.md"), "# Rust Skill\n")?;

        let first = run_in_root(&root, &strings(&["apply"]))?;
        let second = run_in_root(&root, &strings(&["apply"]))?;
        let agents = fs::read_to_string(root.join("AGENTS.md")).map_err(project_io_error)?;

        assert!(first.contains("updated AGENTS.md"));
        assert!(second.contains("unchanged AGENTS.md"));
        assert_eq!(agents.matches(SKILLS_BEGIN_MARKER).count(), 1);
        assert_eq!(agents.matches("# Rust Skill").count(), 1);

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn apply_preserves_content_outside_managed_section() -> ProjectResult<()> {
        let root = test_root("apply-preserve")?;
        let agents = "# AGENTS.md\n\n## Existing\n\nKeep this.\n".to_string();
        write_test_file(&root.join("AGENTS.md"), &agents)?;
        write_test_file(
            &root.join(".codex-skils/skills/testing.md"),
            "# Testing Skill\n",
        )?;

        run_in_root(&root, &strings(&["apply"]))?;
        let agents = fs::read_to_string(root.join("AGENTS.md")).map_err(project_io_error)?;

        assert!(agents.contains("# AGENTS.md"));
        assert!(agents.contains("## Existing\n\nKeep this."));
        assert!(agents.contains("# Testing Skill"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn apply_dry_run_writes_nothing() -> ProjectResult<()> {
        let root = test_root("apply-dry-run")?;
        write_test_file(&root.join(".codex-skils/skills/rust.md"), "# Rust Skill\n")?;

        let output = run_in_root(&root, &strings(&["apply", "--dry-run"]))?;

        assert!(output.contains("found 1 skill(s)"));
        assert!(output.contains("would update AGENTS.md"));
        assert!(output.contains("dry run: no files changed"));
        assert!(!root.join("AGENTS.md").exists());

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn apply_readme_updates_readme_managed_section() -> ProjectResult<()> {
        let root = test_root("apply-readme")?;
        write_test_file(
            &root.join(".codex-skils/skills/security.md"),
            "# Security Skill\n",
        )?;
        write_test_file(
            &root.join(".codex-skils/skills/testing.md"),
            "# Testing Skill\n",
        )?;

        let output = run_in_root(&root, &strings(&["apply", "--readme"]))?;
        let readme = fs::read_to_string(root.join("README.md")).map_err(project_io_error)?;

        assert!(output.contains("updated README.md"));
        assert!(readme.contains(README_RULES_BEGIN_MARKER));
        assert!(readme.contains("## Development Rules"));
        assert!(readme.contains("This project uses codex-skils"));
        assert!(readme.contains("- security"));
        assert!(readme.contains("- testing"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn malformed_markers_fail_without_force() -> ProjectResult<()> {
        let root = test_root("apply-malformed")?;
        write_test_file(
            &root.join("AGENTS.md"),
            &format!("{SKILLS_BEGIN_MARKER}\nold\n"),
        )?;
        write_test_file(&root.join(".codex-skils/skills/rust.md"), "# Rust Skill\n")?;

        let error = run_in_root(&root, &strings(&["apply"]))
            .err()
            .ok_or_else(|| {
                ProjectError::InvalidConfiguration("expected malformed marker error".to_string())
            })?;

        assert!(error
            .to_string()
            .contains("malformed managed section markers"));

        cleanup(&root)?;
        Ok(())
    }

    #[test]
    fn malformed_markers_recover_with_force() -> ProjectResult<()> {
        let root = test_root("apply-malformed-force")?;
        write_test_file(
            &root.join("AGENTS.md"),
            &format!("# AGENTS.md\n{SKILLS_BEGIN_MARKER}\nold\n"),
        )?;
        write_test_file(&root.join(".codex-skils/skills/rust.md"), "# Rust Skill\n")?;

        let output = run_in_root(&root, &strings(&["apply", "--force"]))?;
        let agents = fs::read_to_string(root.join("AGENTS.md")).map_err(project_io_error)?;

        assert!(output.contains("updated AGENTS.md (malformed managed section rebuilt)"));
        assert!(agents.contains("# AGENTS.md"));
        assert!(agents.contains(SKILLS_END_MARKER));
        assert!(agents.contains("# Rust Skill"));

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
