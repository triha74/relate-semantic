mod common;

use common::*;
use serde_json::json;

#[test]
fn remember_writes_generation_one_and_repeats_cleanly() {
    let store = Store::new();
    assert!(!store.exists());
    let first = store.price();
    insta::assert_json_snapshot!(redact(&first));
    assert_eq!(first["at"], T0);
    assert!(store.exists());
    store.set_now("2026-11-11T09:00:00Z");
    let again = store.price();
    assert_eq!(first, again);
}

#[test]
fn remember_a_different_claim_is_key_taken() {
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
            "nw-2024",
            "--about",
            "discounts",
            "--scope",
            "org",
            "--because",
            PRICE_SRC,
        ],
        4,
    );
    insta::assert_json_snapshot!(err);
}

#[test]
fn remember_refuses_bad_input_with_every_problem() {
    let store = Store::new();
    let err = store.fail(
        &[
            "remember",
            "Widget Price",
            "--claim",
            "",
            "--about",
            "North Wind",
            "--about",
            "North Wind",
            "--scope",
            "Org",
            "--because",
            PRICE_SRC,
        ],
        4,
    );
    insta::assert_json_snapshot!(err);
    assert!(!store.exists() || store.run(&[])["scopes"] == json!([]));
}

#[test]
fn remember_refuses_a_claim_over_600_characters() {
    let store = Store::new();
    let claim = "é".repeat(601);
    let err = store.fail(
        &[
            "remember",
            "k",
            "--claim",
            &claim,
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
    assert_eq!(err["error"]["errors"][0]["field"], "claim");
    let claim = "é".repeat(600);
    store.run(&[
        "remember",
        "k",
        "--claim",
        &claim,
        "--about",
        "a",
        "--scope",
        "org",
        "--because",
        "x",
    ]);
}

#[test]
fn remember_needs_the_required_flags() {
    let store = Store::new();
    let err = store.fail(
        &[
            "remember",
            "k",
            "--claim",
            "c",
            "--scope",
            "org",
            "--because",
            "x",
        ],
        2,
    );
    assert_eq!(err["error"]["code"], "usage");
    assert!(err["error"]["message"]
        .as_str()
        .unwrap()
        .contains("--about"));
    assert_eq!(
        err["error"]["fix"],
        "relate-semantic remember KEY --claim TEXT --about SUBJ... --scope SCOPE --because TEXT [--agent NAME]"
    );
    store.set_agent(None);
    let err = store.fail(
        &[
            "remember",
            "k",
            "--claim",
            "c",
            "--about",
            "a",
            "--scope",
            "org",
            "--because",
            "x",
        ],
        2,
    );
    assert_eq!(err["error"]["message"], "remember requires --agent");
    assert_eq!(
        err["error"]["fix"],
        "pass --agent NAME or set RELATE_SEMANTIC_AGENT"
    );
    store.run(&[
        "remember",
        "k",
        "--claim",
        "c",
        "--about",
        "a",
        "--scope",
        "org",
        "--because",
        "x",
        "--agent",
        "invoices",
    ]);
}

#[test]
fn the_database_flag_and_variable_both_work() {
    let store = Store::new();
    let mut cmd = bin();
    cmd.args([
        "-d",
        &store.path(),
        "remember",
        "k",
        "--claim",
        "c",
        "--about",
        "a",
        "--scope",
        "org",
    ])
    .args(["--because", "x", "--agent", "invoices"]);
    cmd.assert().success();
    let mut cmd = bin();
    let out = cmd
        .args(["show", "k"])
        .env("RELATE_SEMANTIC_DATABASE", store.path())
        .output()
        .unwrap();
    assert_eq!(parse(&out.stdout)["memories"][0]["claim"], "c");
}

#[test]
fn a_bad_clock_is_a_usage_error() {
    let store = Store::new();
    store.set_now("yesterday");
    let err = store.fail(&["recall", "--about", "a"], 2);
    assert!(err["error"]["message"]
        .as_str()
        .unwrap()
        .contains("RELATE_SEMANTIC_NOW"));
}

#[test]
fn help_and_version_are_plain_text() {
    let out = bin().arg("--help").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("remember"));
    assert!(text.contains("With no verb"));
    let out = bin().arg("--version").output().unwrap();
    assert!(String::from_utf8(out.stdout)
        .unwrap()
        .starts_with("relate-semantic "));
    let out = bin().args(["remember", "--help"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn an_unknown_verb_is_usage_with_the_top_synopsis() {
    let err = Store::new().fail(&["frobnicate"], 2);
    assert_eq!(err["error"]["code"], "usage");
    assert!(err["error"]["fix"]
        .as_str()
        .unwrap()
        .starts_with("relate-semantic [-d PATH]"));
}
