//! Install the embedded agent skill. The bundle is compiled into the binary.
//! Nothing is read from the source tree at install time.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

const SKILL_NAME: &str = "relate-semantic";
const MARKER: &str = ".relate-semantic-installed";
const VERSION: &str = env!("CARGO_PKG_VERSION");

const BUNDLE: &[(&str, &str)] = &[
    ("SKILL.md", include_str!("skill/bundle/SKILL.md")),
    (
        "references/what-to-remember.md",
        include_str!("skill/bundle/references/what-to-remember.md"),
    ),
    (
        "references/invoice.md",
        include_str!("skill/bundle/references/invoice.md"),
    ),
    (
        "references/scenarios.md",
        include_str!("skill/bundle/references/scenarios.md"),
    ),
    (
        "references/errors.md",
        include_str!("skill/bundle/references/errors.md"),
    ),
];

struct Agent {
    name: &'static str,
    display: &'static str,
    detect: &'static str,
    skills: &'static str,
}

const AGENTS: &[Agent] = &[
    Agent {
        name: "claude-code",
        display: "Claude Code",
        detect: "~/.claude",
        skills: "~/.claude/skills",
    },
    Agent {
        name: "cursor",
        display: "Cursor",
        detect: "~/.cursor",
        skills: "~/.cursor/skills",
    },
    Agent {
        name: "windsurf",
        display: "Windsurf",
        detect: "~/.codeium/windsurf",
        skills: "~/.codeium/windsurf/skills",
    },
    Agent {
        name: "copilot",
        display: "Copilot",
        detect: "~/.copilot",
        skills: "~/.copilot/skills",
    },
    Agent {
        name: "antigravity",
        display: "Antigravity",
        detect: "~/.gemini/antigravity",
        skills: "~/.gemini/antigravity/skills",
    },
    Agent {
        name: "gemini-cli",
        display: "Gemini CLI",
        detect: "~/.gemini",
        skills: "~/.gemini/skills",
    },
    Agent {
        name: "cline",
        display: "Cline",
        detect: "~/.cline",
        skills: "~/.agents/skills",
    },
    Agent {
        name: "codex",
        display: "Codex",
        detect: "~/.codex",
        skills: "~/.codex/skills",
    },
    Agent {
        name: "pi",
        display: "Pi",
        detect: "~/.pi/agent",
        skills: "~/.pi/agent/skills",
    },
    Agent {
        name: "opencode",
        display: "OpenCode",
        detect: "$XDG_CONFIG_HOME/opencode",
        skills: "$XDG_CONFIG_HOME/opencode/skills",
    },
    Agent {
        name: "junie",
        display: "Junie",
        detect: "~/.junie",
        skills: "~/.junie/skills",
    },
];

#[derive(Clone, Copy)]
enum DirState {
    Missing,
    Owned,
    Legacy,
    Foreign,
}

pub fn install(agent: Option<&str>, force: bool) -> i32 {
    mutate(agent, force, true)
}

pub fn remove(agent: Option<&str>, force: bool) -> i32 {
    mutate(agent, force, false)
}

pub fn list(agent: Option<&str>) -> i32 {
    let targets = match targets(agent) {
        Ok(targets) => targets,
        Err(code) => return code,
    };
    let rows: Vec<Value> = targets
        .iter()
        .map(|agent| {
            let dir = agent.skills_dir().join(SKILL_NAME);
            let state = classify(&dir);
            let mut row = json!({
                "name": agent.name,
                "display": agent.display,
                "state": state_name(state),
            });
            if let Some(version) = read_version(&dir.join("SKILL.md")) {
                row["version"] = json!(version);
            }
            row
        })
        .collect();
    crate::output::emit(&json!({"skill": SKILL_NAME, "agents": rows}));
    0
}

pub fn check(agent: Option<&str>) -> i32 {
    let targets = match targets(agent) {
        Ok(targets) => targets,
        Err(code) => return code,
    };
    let mut drifted = Vec::new();
    for agent in &targets {
        let dir = agent.skills_dir().join(SKILL_NAME);
        match classify(&dir) {
            DirState::Owned | DirState::Legacy => {
                if let Err(message) = matches_bundle(&dir) {
                    drifted.push(json!({"name": agent.name, "message": message}));
                }
            }
            DirState::Missing | DirState::Foreign => {}
        }
    }
    if drifted.is_empty() {
        crate::output::emit(&json!({
            "skill": SKILL_NAME,
            "version": VERSION,
            "drift": false,
        }));
        0
    } else {
        fail(
            "skill_drift",
            "an installed skill does not match this binary",
            "relate-semantic skill install",
            false,
            4,
            Some(json!(drifted)),
        )
    }
}

fn mutate(agent: Option<&str>, force: bool, installing: bool) -> i32 {
    let targets = match targets(agent) {
        Ok(targets) => targets,
        Err(code) => return code,
    };
    let mut names = Vec::new();
    for agent in &targets {
        let dir = agent.skills_dir().join(SKILL_NAME);
        let state = classify(&dir);
        if installing {
            if !permits_install(state) && !force {
                return fail(
                    "foreign_skill",
                    format!(
                        "{} already has a skill directory that relate-semantic does not own",
                        agent.display
                    ),
                    format!(
                        "relate-semantic skill install --agent {} --force",
                        agent.name
                    ),
                    false,
                    4,
                    None,
                );
            }
            if let Err(err) = write_bundle(&dir) {
                return io_err(err);
            }
        } else if !permits_remove(state, force) {
            if matches!(state, DirState::Missing) {
                continue;
            }
            return fail(
                "foreign_skill",
                format!("{} is not a relate-semantic skill install", dir.display()),
                "leave the directory in place",
                false,
                4,
                None,
            );
        } else if matches!(state, DirState::Missing) {
            continue;
        } else if let Err(err) = fs::remove_dir_all(&dir) {
            return io_err(err);
        }
        names.push(agent.name);
    }
    crate::output::emit(&json!({
        "skill": SKILL_NAME,
        "version": VERSION,
        "agents": names,
    }));
    0
}

fn targets(agent: Option<&str>) -> Result<Vec<&'static Agent>, i32> {
    if let Some(name) = agent {
        let Some(found) = AGENTS.iter().find(|agent| agent.name == name) else {
            return Err(fail(
                "unknown_agent",
                format!("unknown agent '{name}'"),
                format!(
                    "pass one of {}",
                    AGENTS.iter().map(|a| a.name).collect::<Vec<_>>().join(", ")
                ),
                false,
                4,
                None,
            ));
        };
        return Ok(vec![found]);
    }
    let detected: Vec<&Agent> = AGENTS
        .iter()
        .filter(|agent| agent.detect_dir().is_dir())
        .collect();
    if detected.is_empty() {
        return Err(fail(
            "no_agent",
            "no skill-capable agent was detected",
            "pass --agent NAME",
            false,
            4,
            None,
        ));
    }
    Ok(detected)
}

impl Agent {
    fn detect_dir(&self) -> PathBuf {
        expand(self.detect).unwrap_or_else(|| PathBuf::from(self.detect))
    }

    fn skills_dir(&self) -> PathBuf {
        expand(self.skills).unwrap_or_else(|| PathBuf::from(self.skills))
    }
}

fn classify(dir: &Path) -> DirState {
    let meta = match fs::symlink_metadata(dir) {
        Ok(meta) => meta,
        Err(_) => return DirState::Missing,
    };
    if !meta.is_dir() {
        return DirState::Foreign;
    }
    let marker = dir.join(MARKER);
    match fs::symlink_metadata(&marker) {
        Ok(meta) if meta.file_type().is_symlink() => return DirState::Foreign,
        Ok(meta) if meta.is_file() => return DirState::Owned,
        _ => {}
    }
    if skill_name_is_ours(&dir.join("SKILL.md")) {
        DirState::Legacy
    } else {
        DirState::Foreign
    }
}

fn permits_install(state: DirState) -> bool {
    matches!(
        state,
        DirState::Missing | DirState::Owned | DirState::Legacy
    )
}

fn permits_remove(state: DirState, force: bool) -> bool {
    match state {
        DirState::Owned => true,
        DirState::Legacy => force,
        DirState::Missing => true,
        DirState::Foreign => false,
    }
}

fn write_bundle(dir: &Path) -> io::Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)?;
    for (rel, contents) in BUNDLE {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = contents.replace("{{VERSION}}", VERSION);
        fs::write(path, text)?;
    }
    fs::write(dir.join(MARKER), VERSION)?;
    Ok(())
}

fn matches_bundle(dir: &Path) -> Result<(), String> {
    for (rel, contents) in BUNDLE {
        let expected = contents.replace("{{VERSION}}", VERSION);
        let path = dir.join(rel);
        let actual =
            fs::read_to_string(&path).map_err(|err| format!("{}: {err}", path.display()))?;
        if actual != expected {
            return Err(format!("{rel} does not match version {VERSION}"));
        }
    }
    Ok(())
}

fn skill_name_is_ours(path: &Path) -> bool {
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    frontmatter_value(&text, "name").is_some_and(|name| name == SKILL_NAME)
}

fn read_version(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    frontmatter_value(&text, "version")
}

fn frontmatter_value(text: &str, key: &str) -> Option<String> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    for line in rest[..end].lines() {
        if let Some(value) = line.strip_prefix(&format!("{key}:")) {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn state_name(state: DirState) -> &'static str {
    match state {
        DirState::Missing => "missing",
        DirState::Owned | DirState::Legacy => "installed",
        DirState::Foreign => "foreign",
    }
}

fn expand(path: &str) -> Option<PathBuf> {
    if let Some(rest) = path.strip_prefix("$XDG_CONFIG_HOME") {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .or_else(|| home().map(|h| h.join(".config")))?;
        return Some(base.join(rest.trim_start_matches('/')));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return Some(home()?.join(rest));
    }
    if path == "~" {
        return home();
    }
    Some(PathBuf::from(path))
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn io_err(err: io::Error) -> i32 {
    fail(
        "skill_io",
        err.to_string(),
        "check that the skills directory is writable",
        false,
        1,
        None,
    )
}

fn fail(
    code: &str,
    message: impl Into<String>,
    fix: impl Into<String>,
    retryable: bool,
    exit: i32,
    extra: Option<Value>,
) -> i32 {
    let mut body = json!({
        "code": code,
        "message": message.into(),
        "fix": fix.into(),
        "retryable": retryable,
    });
    if let Some(extra) = extra {
        body["errors"] = extra;
    }
    crate::output::emit(&json!({"error": body}));
    exit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_lists_every_shipped_file() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/skill/bundle");
        let mut found = Vec::new();
        walk(&root, &root, &mut found);
        found.sort();
        let mut listed: Vec<&str> = BUNDLE.iter().map(|(name, _)| *name).collect();
        listed.sort();
        assert_eq!(found, listed);
    }

    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push(rel);
            }
        }
    }

    #[test]
    fn install_adopts_legacy_lists_and_refuses_a_file() {
        let home = tempfile::tempdir().unwrap();
        let prior = std::env::var_os("HOME");
        std::env::set_var("HOME", home.path());
        let cursor = home.path().join(".cursor");
        fs::create_dir_all(cursor.join("skills")).unwrap();
        assert_eq!(list(None), 0);
        let legacy = cursor.join("skills/relate-semantic");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(
            legacy.join("SKILL.md"),
            "---\nname: relate-semantic\nversion: old\n---\n",
        )
        .unwrap();
        assert_eq!(install(Some("cursor"), false), 0);
        assert!(legacy.join(MARKER).is_file());
        assert_eq!(check(Some("cursor")), 0);
        assert_eq!(remove(Some("cursor"), false), 0);
        assert_eq!(remove(Some("cursor"), false), 0);
        fs::remove_dir_all(cursor.join("skills")).unwrap();
        fs::write(cursor.join("skills"), "not a directory").unwrap();
        assert_eq!(install(Some("cursor"), false), 1);
        if let Some(prior) = prior {
            std::env::set_var("HOME", prior);
        }
    }

    #[test]
    fn version_placeholder_is_replaced() {
        let text = BUNDLE[0].1;
        assert!(text.contains("{{VERSION}}"));
        assert!(text.contains("name: relate-semantic"));
    }
}
