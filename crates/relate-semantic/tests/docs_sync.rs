//! The docs are part of the contract. These tests fail when the binary and
//! `docs/cli.md`, the skill's `references/errors.md`, or a JSON example in
//! any doc disagree.

mod common;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use common::*;
use relate_semantic_core::{exit_for, ERROR_CODES, RECALL_ACTIONS};

const VERBS: &[&str] = &[
    "recall", "remember", "correct", "forget", "show", "log", "prune", "skill",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    fs::read_to_string(root().join(rel)).unwrap_or_else(|err| panic!("{rel}: {err}"))
}

fn cli_md() -> String {
    read("docs/cli.md")
}

/// Rows of every table whose header starts with `| Code |`: (code, exit).
fn code_rows(text: &str) -> Vec<(String, u8)> {
    let mut rows = Vec::new();
    let mut in_table = false;
    for line in text.lines() {
        if line.starts_with("| Code |") {
            in_table = true;
            continue;
        }
        if !line.starts_with('|') {
            in_table = false;
            continue;
        }
        if !in_table || line.starts_with("| ---") {
            continue;
        }
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        let code = cells[1].trim_matches('`').to_string();
        let exit: u8 = cells[2]
            .parse()
            .unwrap_or_else(|_| panic!("exit cell in {line}"));
        rows.push((code, exit));
    }
    rows
}

/// The text of `### heading` up to the next `##` or `###` heading.
fn section<'a>(text: &'a str, heading: &str) -> &'a str {
    let marker = format!("\n### {heading}\n");
    let start = text
        .find(&marker)
        .unwrap_or_else(|| panic!("docs/cli.md has no ### {heading} section"))
        + marker.len();
    let rest = &text[start..];
    let end = rest
        .find("\n## ")
        .into_iter()
        .chain(rest.find("\n### "))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

fn check_codes(doc: &str, text: &str) {
    let rows = code_rows(text);
    let documented: BTreeSet<&str> = rows.iter().map(|(c, _)| c.as_str()).collect();
    for code in ERROR_CODES {
        assert!(documented.contains(code), "{doc} has no row for {code}");
    }
    for (code, exit) in &rows {
        let expected = exit_for(code)
            .unwrap_or_else(|| panic!("{doc} documents {code}, which is not in ERROR_CODES"));
        assert_eq!(
            *exit, expected,
            "{doc} says {code} exits {exit}, the binary exits {expected}"
        );
    }
}

#[test]
fn every_error_code_is_documented_in_cli_md_with_its_exit() {
    check_codes("docs/cli.md", &cli_md());
}

#[test]
fn every_error_code_is_documented_in_the_skill_with_its_exit() {
    check_codes(
        "references/errors.md",
        &read("crates/relate-semantic/src/skill/bundle/references/errors.md"),
    );
}

#[test]
fn the_exit_table_lists_every_exit_the_binary_uses() {
    let text = cli_md();
    let mut exits = BTreeSet::new();
    let mut in_table = false;
    for line in text.lines() {
        if line.starts_with("| Exit |") {
            in_table = true;
            continue;
        }
        if !line.starts_with('|') {
            if in_table {
                break;
            }
            continue;
        }
        if in_table && !line.starts_with("| ---") {
            let cell = line.split('|').nth(1).unwrap().trim();
            exits.insert(cell.parse::<u8>().unwrap());
        }
    }
    let mut expected: BTreeSet<u8> = ERROR_CODES.iter().filter_map(|c| exit_for(c)).collect();
    expected.insert(0);
    assert_eq!(exits, expected);
}

#[test]
fn every_verb_has_a_section_with_its_synopsis() {
    let text = cli_md();
    let store = Store::new();
    for verb in VERBS {
        let body = section(&text, verb);
        let err = store.fail(&[verb, "--not-a-flag"], 2);
        let synopsis = err["error"]["fix"].as_str().unwrap();
        assert!(
            body.contains(synopsis),
            "the {verb} section does not show the synopsis the binary prints:\n{synopsis}"
        );
        assert!(
            text.contains(&format!("{synopsis}\n")),
            "the synopsis block at the top of docs/cli.md is missing:\n{synopsis}"
        );
    }
}

fn flags_in_help(args: &[&str]) -> BTreeSet<String> {
    let out = bin().args(args).arg("--help").output().unwrap();
    let help = String::from_utf8(out.stdout).unwrap();
    let mut flags = BTreeSet::new();
    for word in
        help.split(|c: char| c.is_whitespace() || c == ',' || c == '[' || c == ']' || c == '=')
    {
        if let Some(name) = word.strip_prefix("--") {
            let name: String = name
                .chars()
                .take_while(|c| c.is_ascii_lowercase() || *c == '-')
                .collect();
            if !name.is_empty() && name != "help" && name != "version" && name != "database" {
                flags.insert(format!("--{name}"));
            }
        }
    }
    flags
}

#[test]
fn every_flag_the_binary_takes_is_in_its_section() {
    let text = cli_md();
    for verb in VERBS {
        let body = section(&text, verb);
        let mut flags = flags_in_help(&[verb]);
        if *verb == "skill" {
            for action in ["install", "remove", "list", "check"] {
                flags.extend(flags_in_help(&["skill", action]));
            }
        }
        for flag in flags {
            assert!(
                body.contains(&format!("`{flag}")),
                "the {verb} section does not document {flag}"
            );
        }
    }
    let options = &text[text.find("## Options").expect("an Options section")..];
    assert!(options.contains("`--database"));
    assert!(options.contains("RELATE_SEMANTIC_DATABASE"));
}

#[test]
fn every_recall_action_is_documented() {
    let text = cli_md();
    let body = section(&text, "recall");
    for action in RECALL_ACTIONS {
        assert!(
            body.contains(&format!("| `{action}` |")),
            "recall does not document {action}"
        );
    }
}

fn json_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        match &mut current {
            None if line.trim_start() == "```json" => current = Some(String::new()),
            Some(block) if line.trim_start() == "```" => {
                blocks.push(std::mem::take(block));
                current = None;
            }
            Some(block) => {
                block.push_str(line);
                block.push('\n');
            }
            None => {}
        }
    }
    blocks
}

fn markdown_files() -> Vec<PathBuf> {
    let mut files = vec![root().join("DESIGN.md"), root().join("README.md")];
    for dir in [
        "docs",
        "docs/scenarios",
        "crates/relate-semantic/src/skill/bundle",
        "crates/relate-semantic/src/skill/bundle/references",
        "examples/invoice",
        "examples/incident",
        "examples/onboarding",
    ] {
        for entry in fs::read_dir(root().join(dir)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "md") {
                files.push(path);
            }
        }
    }
    files
}

#[test]
fn every_json_example_in_the_docs_parses() {
    let mut count = 0;
    for path in markdown_files() {
        let text = fs::read_to_string(&path).unwrap();
        for block in json_blocks(&text) {
            count += 1;
            if serde_json::from_str::<serde_json::Value>(&block).is_ok() {
                continue;
            }
            for line in block.lines().filter(|l| !l.trim().is_empty()) {
                serde_json::from_str::<serde_json::Value>(line).unwrap_or_else(|err| {
                    panic!(
                        "{}: a json block does not parse: {err}\n{line}",
                        path.display()
                    )
                });
            }
        }
    }
    assert!(count > 20, "only {count} json examples were found");
}
