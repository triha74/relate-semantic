mod support;

use relate_semantic_core::{Memory, Recall, MAX_LIMIT};
use support::*;

#[test]
fn remember_lists_every_problem_and_the_fix_repairs_the_slugs() {
    let memory = open();
    let mut spec = remember(
        "Widget Price",
        "Org",
        "",
        &["North Wind", "discounts", "discounts"],
    );
    spec.because = "x".repeat(700);
    spec.agent = "Invoice Bot".into();
    let err = memory.remember(spec).unwrap_err();
    assert_eq!(err.code, "bad_slug");
    assert_eq!(err.exit_code(), 4);
    let codes: Vec<(&str, Option<&str>)> = err
        .errors
        .iter()
        .map(|p| (p.code, p.field.as_deref()))
        .collect();
    assert_eq!(
        codes,
        vec![
            ("bad_slug", Some("key")),
            ("bad_slug", Some("about")),
            ("duplicate_value", Some("about")),
            ("bad_slug", Some("scope")),
            ("empty_field", Some("claim")),
            ("too_long", Some("because")),
            ("bad_slug", Some("agent")),
        ]
    );
    assert_eq!(
        err.fix,
        "relate-semantic remember widget-price --claim TEXT --about north-wind --about discounts --scope org --because TEXT --agent invoice-bot"
    );
    let body = err.to_json();
    assert_eq!(
        body["error"]["errors"][5]["message"],
        "--because is 700 characters and the limit is 600"
    );
}

#[test]
fn remember_without_subjects_is_empty_field() {
    let err = open()
        .remember(remember("widget-price", "org", PRICE, &[]))
        .unwrap_err();
    assert_eq!(err.code, "empty_field");
    assert_eq!(err.message, "remember needs at least one --about");
    assert_eq!(
        err.fix,
        "relate-semantic remember widget-price --claim TEXT --about SUBJ --scope org --because TEXT"
    );
}

#[test]
fn a_slug_with_no_sensible_suggestion_becomes_a_placeholder() {
    let err = open()
        .remember(remember("2024", "!!", PRICE, &["9"]))
        .unwrap_err();
    assert_eq!(
        err.fix,
        "relate-semantic remember KEY --claim TEXT --about SUBJ --scope SCOPE --because TEXT"
    );
}

#[test]
fn correct_and_forget_validate_every_field() {
    let memory = open();
    let mut spec = correct("Widget", "org", " ", Some(&["A", "A"]));
    spec.because = "".into();
    let err = memory.correct(spec).unwrap_err();
    let codes: Vec<&str> = err.errors.iter().map(|p| p.code).collect();
    assert_eq!(
        codes,
        vec![
            "bad_slug",
            "empty_field",
            "empty_field",
            "bad_slug",
            "duplicate_value"
        ]
    );
    assert_eq!(
        err.fix,
        "relate-semantic correct widget --scope org --claim TEXT --because TEXT --about a"
    );

    let mut spec = forget("k", "Case 7");
    spec.because = "   ".into();
    let err = memory.forget(spec).unwrap_err();
    assert_eq!(err.code, "bad_slug");
    assert_eq!(err.errors[1].code, "empty_field");
    assert_eq!(
        err.fix,
        "relate-semantic forget k --scope case-7 --because TEXT"
    );
}

#[test]
fn recall_validates_its_slugs_and_its_limit() {
    let memory = open();
    let err = memory
        .recall(recall(&["North Wind"], &["Org", "org"]))
        .unwrap_err();
    assert_eq!(err.code, "bad_slug");
    assert_eq!(
        err.fix,
        "relate-semantic recall --about north-wind --scope org"
    );

    let err = memory.recall(recall(&["a"], &["org", "org"])).unwrap_err();
    assert_eq!(err.code, "duplicate_value");

    let err = memory.recall(recall_keys(&["Key"], &[])).unwrap_err();
    assert_eq!(err.fix, "relate-semantic recall --key key");

    let err = memory.recall(recall(&[], &["org"])).unwrap_err();
    assert_eq!(err.code, "usage");
    assert_eq!(err.exit_code(), 2);
    assert_eq!(err.fix, "relate-semantic recall --scope org --about SUBJ");

    for limit in [0, MAX_LIMIT + 1] {
        let err = memory
            .recall(Recall {
                limit,
                ..recall(&["a"], &[])
            })
            .unwrap_err();
        assert_eq!(err.code, "usage");
        assert!(err.message.contains("--limit"));
    }
}

#[test]
fn show_log_and_prune_validate_their_slugs() {
    let memory = open();
    assert_eq!(
        memory.show("Bad Key").unwrap_err().fix,
        "relate-semantic show bad-key"
    );
    assert_eq!(
        memory.log("Bad Key", Some("Org")).unwrap_err().fix,
        "relate-semantic log bad-key --scope org"
    );
    assert_eq!(
        memory.prune("Org").unwrap_err().fix,
        "relate-semantic prune --scope org"
    );
}

#[test]
fn show_and_log_on_a_missing_key_are_no_memory() {
    let memory = open();
    let err = memory.show("widget-price").unwrap_err();
    assert_eq!(err.code, "no_memory");
    assert_eq!(err.message, "widget-price is not live in any scope");
    assert_eq!(
        err.fix,
        "relate-semantic remember widget-price --claim TEXT --about SUBJ --scope SCOPE --because TEXT"
    );
    let err = memory.log("widget-price", None).unwrap_err();
    assert_eq!(err.message, "widget-price has no generations");
    let err = memory.log("widget-price", Some("org")).unwrap_err();
    assert_eq!(err.message, "widget-price has no generations in org");
    assert!(err.fix.contains("--scope org"));
}

#[test]
fn a_foreign_graph_is_refused_for_reads_and_writes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db");
    {
        let db = relate::Database::open(path.to_str().unwrap()).unwrap();
        db.connect()
            .unwrap()
            .query("CREATE (:Widget {name: 'invoice'})")
            .unwrap();
    }
    let memory = open_at(path.to_str().unwrap());
    let err = memory.recall(recall(&["a"], &[])).unwrap_err();
    assert_eq!(err.code, "foreign_store");
    assert_eq!(err.exit_code(), 4);
    let err = memory
        .remember(remember("k", "org", "c", &["a"]))
        .unwrap_err();
    assert_eq!(err.code, "foreign_store");
}

#[test]
fn a_relate_flow_graph_recalls_as_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db");
    {
        let db = relate::Database::open(path.to_str().unwrap()).unwrap();
        db.connect()
            .unwrap()
            .query("CREATE (:Plan {name: 'invoice'})")
            .unwrap();
    }
    let memory = open_at(path.to_str().unwrap());
    let out = memory.recall(recall(&["northwind"], &["org"])).unwrap();
    assert_eq!(out["action"], "none");
}

#[test]
fn a_path_that_is_a_file_is_a_store_error() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("file");
    std::fs::write(&file, "not a store").unwrap();
    let err = Memory::open(file.to_str().unwrap()).err().unwrap();
    assert_eq!(err.code, "store");
    assert_eq!(err.exit_code(), 1);
    assert!(!err.retryable);
}

#[test]
fn a_second_opener_is_store_busy() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db");
    let first = Memory::open(path.to_str().unwrap()).unwrap();
    first.remember(remember("k", "org", "c", &["a"])).unwrap();
    match Memory::open(path.to_str().unwrap()) {
        Err(err) => {
            assert_eq!(err.code, "store_busy");
            assert_eq!(err.exit_code(), 5);
            assert!(err.retryable);
        }
        Ok(second) => {
            // Relate may share one engine per process. Then both openers see the same store.
            assert_eq!(second.show("k").unwrap()["memories"][0]["claim"], "c");
        }
    }
}
