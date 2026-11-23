mod common;

use std::fs;

use common::bin;

#[test]
fn install_list_check_and_remove_round_trip() {
    let home = tempfile::tempdir().unwrap();
    let installed = skill(&home, &["skill", "install", "--agent", "cursor"]);
    insta::assert_json_snapshot!(installed);
    let dir = home.path().join(".cursor/skills/relate-semantic");
    let skill_md = dir.join("SKILL.md");
    let text = fs::read_to_string(&skill_md).unwrap();
    assert!(text.contains("version: 0.1.0"));
    assert!(text.contains("Recall before you act."));
    for reference in ["what-to-remember", "errors", "invoice", "scenarios"] {
        assert!(
            dir.join(format!("references/{reference}.md")).is_file(),
            "{reference}"
        );
    }
    assert!(dir.join(".relate-semantic-installed").is_file());

    let listed = skill(&home, &["skill", "list", "--agent", "cursor"]);
    assert_eq!(listed["agents"][0]["state"], "installed");
    assert_eq!(listed["agents"][0]["version"], "0.1.0");
    let checked = skill(&home, &["skill", "check", "--agent", "cursor"]);
    assert_eq!(checked["drift"], false);

    fs::write(&skill_md, "---\nname: relate-semantic\nversion: 9\n---\n").unwrap();
    let drift = skill_fail(&home, &["skill", "check", "--agent", "cursor"], 4);
    assert_eq!(drift["error"]["code"], "skill_drift");
    assert_eq!(drift["error"]["fix"], "relate-semantic skill install");

    skill(&home, &["skill", "install", "--agent", "cursor"]);
    let removed = skill(&home, &["skill", "remove", "--agent", "cursor"]);
    assert_eq!(removed["agents"], serde_json::json!(["cursor"]));
    assert!(!skill_md.exists());
    let again = skill(&home, &["skill", "remove", "--agent", "cursor"]);
    assert_eq!(again["agents"], serde_json::json!([]));
}

#[test]
fn every_detected_agent_is_installed_without_agent() {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join(".cursor")).unwrap();
    fs::create_dir_all(home.path().join(".claude")).unwrap();
    let installed = skill(&home, &["skill", "install"]);
    assert_eq!(
        installed["agents"],
        serde_json::json!(["claude-code", "cursor"])
    );
    let listed = skill(&home, &["skill", "list"]);
    assert_eq!(listed["agents"].as_array().unwrap().len(), 2);
}

#[test]
fn a_foreign_directory_is_refused_until_force() {
    let home = tempfile::tempdir().unwrap();
    let dir = home.path().join(".cursor/skills/relate-semantic");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("SKILL.md"), "---\nname: other\n---\n").unwrap();
    let refused = skill_fail(&home, &["skill", "install", "--agent", "cursor"], 4);
    assert_eq!(refused["error"]["code"], "foreign_skill");
    assert_eq!(
        refused["error"]["fix"],
        "relate-semantic skill install --agent cursor --force"
    );
    let listed = skill(&home, &["skill", "list", "--agent", "cursor"]);
    assert_eq!(listed["agents"][0]["state"], "foreign");
    skill(&home, &["skill", "install", "--agent", "cursor", "--force"]);
    assert!(dir.join(".relate-semantic-installed").is_file());
    fs::write(dir.join("SKILL.md"), "---\nname: other\n---\n").unwrap();
    fs::remove_file(dir.join(".relate-semantic-installed")).unwrap();
    let still = skill_fail(&home, &["skill", "remove", "--agent", "cursor"], 4);
    assert_eq!(still["error"]["code"], "foreign_skill");
    let forced = skill_fail(
        &home,
        &["skill", "remove", "--agent", "cursor", "--force"],
        4,
    );
    assert_eq!(forced["error"]["fix"], "leave the directory in place");
    assert!(
        dir.join("SKILL.md").is_file(),
        "remove never deletes someone else's directory"
    );
}

#[test]
fn force_removes_a_copy_that_lost_its_marker() {
    let home = tempfile::tempdir().unwrap();
    skill(&home, &["skill", "install", "--agent", "cursor"]);
    let dir = home.path().join(".cursor/skills/relate-semantic");
    fs::remove_file(dir.join(".relate-semantic-installed")).unwrap();
    let refused = skill_fail(&home, &["skill", "remove", "--agent", "cursor"], 4);
    assert_eq!(refused["error"]["code"], "foreign_skill");
    skill(&home, &["skill", "remove", "--agent", "cursor", "--force"]);
    assert!(!dir.exists());
}

#[test]
fn unknown_and_missing_agents_are_refused() {
    let home = tempfile::tempdir().unwrap();
    let unknown = skill_fail(&home, &["skill", "install", "--agent", "nope"], 4);
    assert_eq!(unknown["error"]["code"], "unknown_agent");
    assert!(unknown["error"]["fix"].as_str().unwrap().contains("cursor"));
    let none = skill_fail(&home, &["skill", "list"], 4);
    assert_eq!(none["error"]["code"], "no_agent");
    assert_eq!(none["error"]["fix"], "pass --agent NAME");
}

#[test]
fn an_unwritable_skills_directory_is_skill_io() {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join(".cursor")).unwrap();
    fs::write(
        home.path().join(".cursor/skills"),
        "a file, not a directory",
    )
    .unwrap();
    let err = skill_fail(&home, &["skill", "install", "--agent", "cursor"], 1);
    assert_eq!(err["error"]["code"], "skill_io");
    assert_eq!(err["error"]["retryable"], false);
}

#[test]
fn skill_needs_an_action() {
    let home = tempfile::tempdir().unwrap();
    let err = skill_fail(&home, &["skill", "frob"], 2);
    assert_eq!(
        err["error"]["fix"],
        "relate-semantic skill install|remove|list|check [--agent NAME] [--force]"
    );
}

fn skill(home: &tempfile::TempDir, args: &[&str]) -> serde_json::Value {
    let out = bin()
        .args(args)
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join(".config"))
        .assert()
        .success();
    common::parse(&out.get_output().stdout)
}

fn skill_fail(home: &tempfile::TempDir, args: &[&str], code: i32) -> serde_json::Value {
    let out = bin()
        .args(args)
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join(".config"))
        .assert()
        .failure()
        .code(code);
    common::parse(&out.get_output().stdout)
}
