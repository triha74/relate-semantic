//! Every verb, every library error code, and every recall action is named by a test.

use relate_semantic_core::{ERROR_CODES, RECALL_ACTIONS};
use std::fs;
use std::path::Path;

pub const VERBS: &[&str] = &[
    "recall", "remember", "correct", "forget", "show", "log", "prune", "skill",
];

#[test]
fn every_verb_error_code_and_recall_action_has_a_test() {
    let mut sources = String::new();
    for root in [
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../relate-semantic-core/tests"),
    ] {
        walk(&root, &mut sources);
    }
    let this = include_str!("matrix.rs");
    let sources = sources.replace(this, "");
    for verb in VERBS {
        assert!(
            sources.contains(&format!("\"{verb}\"")),
            "no test runs {verb}"
        );
    }
    for code in ERROR_CODES {
        assert!(
            sources.contains(&format!("\"{code}\"")),
            "no test mentions {code}"
        );
    }
    for action in RECALL_ACTIONS {
        assert!(
            sources.contains(&format!("\"{action}\"")),
            "no test expects the {action} action"
        );
    }
}

fn walk(dir: &Path, out: &mut String) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push_str(&fs::read_to_string(&path).unwrap());
        }
    }
}
