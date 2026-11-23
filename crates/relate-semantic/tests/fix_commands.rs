//! Every refusal's `fix` is a command that works. Each case triggers one
//! refusal through the binary, fills in the placeholders, and runs the fix
//! against the same store.

mod common;

use common::*;
use relate_semantic_core::{exit_for, fix::PLACEHOLDERS, ERROR_CODES};
use serde_json::Value;

/// Refusals whose `fix` is a sentence, because the agent has to choose a value
/// the refused call did not contain. Every other exit-4 code must be covered by
/// a case below.
const SENTENCE_FIXES: &[&str] = &["foreign_store", "unknown_agent", "no_agent"];

fn token(word: &str) -> String {
    match word {
        "TEXT" => "a fact the next agent can use".into(),
        "SUBJ" => "northwind".into(),
        "SCOPE" => "org".into(),
        "KEY" => "widget-price".into(),
        "DURATION" => "30d".into(),
        other => other.into(),
    }
}

/// Split a fix into argv. Values are slugs or placeholders, so a fix never
/// needs quoting.
fn argv(fix: &str) -> Vec<String> {
    let words: Vec<&str> = fix.split_whitespace().collect();
    assert_eq!(words.first().copied(), Some("relate-semantic"), "{fix}");
    for word in &words[1..] {
        let upper = word.chars().all(|c| c.is_ascii_uppercase());
        if upper {
            assert!(
                PLACEHOLDERS.contains(word),
                "{word} is not a placeholder in {fix}"
            );
        }
    }
    words[1..].iter().map(|w| token(w)).collect()
}

/// Run the fix. `must_succeed` is true when the fix is the whole repair.
fn follow(store: &Store, error: &Value, must_succeed: bool) -> Value {
    let fix = error["error"]["fix"].as_str().unwrap().to_string();
    let args = argv(&fix);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = store.cmd(&refs).output().unwrap();
    let body = parse(&output.stdout);
    assert_ne!(
        output.status.code(),
        Some(2),
        "fix does not parse: {fix} -> {body}"
    );
    if must_succeed {
        assert_eq!(output.status.code(), Some(0), "fix failed: {fix} -> {body}");
    }
    body
}

fn covered(code: &str, seen: &mut Vec<&'static str>) {
    let found = ERROR_CODES
        .iter()
        .find(|c| **c == code)
        .expect("known code");
    seen.push(found);
}

#[test]
fn every_refusal_fix_runs() {
    let mut seen: Vec<&'static str> = Vec::new();

    let store = Store::new();
    let err = store.fail(
        &[
            "remember",
            "Widget Price",
            "--claim",
            PRICE,
            "--about",
            "North Wind",
            "--about",
            "NW 2024",
            "--scope",
            "Org",
            "--because",
            PRICE_SRC,
        ],
        4,
    );
    assert_eq!(err["error"]["code"], "bad_slug");
    covered("bad_slug", &mut seen);
    let done = follow(&store, &err, true);
    assert_eq!(done["key"], "widget-price");
    assert_eq!(done["scope"], "org");

    let store = Store::new();
    let err = store.fail(
        &[
            "remember",
            "k",
            "--claim",
            " ",
            "--about",
            "a",
            "--scope",
            "org",
            "--because",
            "x",
        ],
        4,
    );
    assert_eq!(err["error"]["code"], "empty_field");
    covered("empty_field", &mut seen);
    follow(&store, &err, true);

    let store = Store::new();
    let long = "x".repeat(601);
    let err = store.fail(
        &[
            "remember",
            "k",
            "--claim",
            &long,
            "--about",
            "a",
            "--scope",
            "org",
            "--because",
            "x",
        ],
        4,
    );
    assert_eq!(err["error"]["code"], "too_long");
    covered("too_long", &mut seen);
    follow(&store, &err, true);

    let store = Store::new();
    let err = store.fail(
        &[
            "remember",
            "k",
            "--claim",
            "c",
            "--about",
            "a",
            "--about",
            "a",
            "--scope",
            "org",
            "--because",
            "x",
        ],
        4,
    );
    assert_eq!(err["error"]["code"], "duplicate_value");
    covered("duplicate_value", &mut seen);
    follow(&store, &err, true);

    let store = Store::new();
    store.price();
    let err = store.fail(
        &[
            "remember",
            "widget-price",
            "--claim",
            PRICE_2025,
            "--about",
            "northwind",
            "--about",
            "nw-2025",
            "--scope",
            "org",
            "--because",
            "new agreement",
        ],
        4,
    );
    assert_eq!(err["error"]["code"], "key_taken");
    covered("key_taken", &mut seen);
    let done = follow(&store, &err, true);
    assert_eq!(done["generation"], 2);
    let shown = store.run(&["show", "widget-price"]);
    assert_eq!(
        shown["memories"][0]["about"],
        serde_json::json!(["northwind", "nw-2025"])
    );

    let store = Store::new();
    let err = store.fail(
        &[
            "correct",
            "widget-price",
            "--scope",
            "org",
            "--claim",
            PRICE,
            "--because",
            "x",
            "--about",
            "northwind",
        ],
        4,
    );
    assert_eq!(err["error"]["code"], "no_memory");
    covered("no_memory", &mut seen);
    follow(&store, &err, true);

    let store = Store::new();
    let err = store.fail(&["show", "widget-price"], 4);
    assert_eq!(err["error"]["code"], "no_memory");
    follow(&store, &err, true);

    let store = Store::new();
    store.price();
    store.run(&[
        "forget",
        "widget-price",
        "--scope",
        "org",
        "--because",
        "gone",
    ]);
    let err = store.fail(&["show", "widget-price"], 4);
    assert_eq!(err["error"]["fix"], "relate-semantic log widget-price");
    follow(&store, &err, true);
    let err = store.fail(
        &[
            "correct",
            "widget-price",
            "--scope",
            "org",
            "--claim",
            PRICE,
            "--because",
            "x",
        ],
        4,
    );
    assert!(err["error"]["message"]
        .as_str()
        .unwrap()
        .contains("forgotten"));
    follow(&store, &err, true);

    let store = Store::new();
    let err = store.fail(&["log", "widget-price", "--scope", "org"], 4);
    assert_eq!(err["error"]["code"], "no_memory");
    follow(&store, &err, true);

    let store = Store::new();
    store.set_agent(Some("Bot 1"));
    let err = store.fail(&["forget", "k", "--scope", "org", "--because", "x"], 4);
    assert_eq!(
        err["error"]["fix"],
        "relate-semantic forget k --scope org --because TEXT --agent bot-1"
    );
    follow(&store, &err, true);

    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(home.path().join(".cursor/skills/relate-semantic")).unwrap();
    std::fs::write(
        home.path().join(".cursor/skills/relate-semantic/SKILL.md"),
        "---\nname: other\n---\n",
    )
    .unwrap();
    let mut cmd = bin();
    let out = cmd
        .args(["skill", "install", "--agent", "cursor"])
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let err = parse(&out.stdout);
    assert_eq!(err["error"]["code"], "foreign_skill");
    covered("foreign_skill", &mut seen);
    let fix = argv(err["error"]["fix"].as_str().unwrap());
    let out = bin().args(&fix).env("HOME", home.path()).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );

    std::fs::write(
        home.path().join(".cursor/skills/relate-semantic/SKILL.md"),
        "---\nname: relate-semantic\nversion: old\n---\n",
    )
    .unwrap();
    let out = bin()
        .args(["skill", "check", "--agent", "cursor"])
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let err = parse(&out.stdout);
    assert_eq!(err["error"]["code"], "skill_drift");
    covered("skill_drift", &mut seen);
    let fix = argv(err["error"]["fix"].as_str().unwrap());
    let out = bin().args(&fix).env("HOME", home.path()).output().unwrap();
    assert_eq!(out.status.code(), Some(0));

    for code in ERROR_CODES {
        if exit_for(code) != Some(4) {
            continue;
        }
        assert!(
            seen.contains(code) || SENTENCE_FIXES.contains(code),
            "{code} has no fix case and is not listed as a sentence fix"
        );
    }
}

#[test]
fn sentence_fixes_are_sentences() {
    let store = Store::new();
    let flow = tempfile::tempdir().unwrap();
    let path = flow.path().join("db");
    {
        let db = relate::Database::open(path.to_str().unwrap()).unwrap();
        let mut conn = db.connect().unwrap();
        conn.query("CREATE (:Widget {name: 'invoice'})").unwrap();
    }
    let out = bin()
        .args([
            "recall",
            "--about",
            "northwind",
            "-d",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let err = parse(&out.stdout);
    assert_eq!(err["error"]["code"], "foreign_store");
    assert!(!err["error"]["fix"]
        .as_str()
        .unwrap()
        .starts_with("relate-semantic"));

    let err = store.fail(&["skill", "list", "--agent", "nope"], 4);
    assert_eq!(err["error"]["code"], "unknown_agent");
    let home = tempfile::tempdir().unwrap();
    let out = bin()
        .args(["skill", "list"])
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join("x"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(parse(&out.stdout)["error"]["code"], "no_agent");
}
