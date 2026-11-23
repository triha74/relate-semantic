mod common;

use common::*;
use serde_json::json;

#[test]
fn recall_use_returns_the_memory() {
    let store = Store::new();
    store.price();
    let out = store.run(&["recall", "--about", "northwind", "--about", "discounts"]);
    insta::assert_json_snapshot!(out);
}

#[test]
fn recall_none_echoes_the_query() {
    let store = Store::new();
    store.price();
    let out = store.run(&["recall", "--about", "acme", "--scope", "org"]);
    assert_eq!(
        out,
        json!({"action": "none", "about": ["acme"], "keys": [], "scopes": ["org"]})
    );
}

#[test]
fn recall_on_a_missing_store_is_none_and_creates_nothing() {
    let store = Store::new();
    let out = store.run(&["recall", "--about", "northwind"]);
    assert_eq!(out["action"], "none");
    assert!(!store.exists());
}

#[test]
fn recall_conflict_until_scopes_are_ordered() {
    let store = Store::new();
    store.price();
    store.run(&[
        "remember",
        "widget-price",
        "--claim",
        "INV-1042 is billed at $90 per widget.",
        "--about",
        "northwind",
        "--scope",
        "case-inv-1042",
        "--because",
        "Approved by finance on 2026-11-10.",
    ]);
    let out = store.run(&["recall", "--about", "northwind"]);
    insta::assert_json_snapshot!(out);
    let fixed: Vec<String> = out["fix"]
        .as_str()
        .unwrap()
        .split(' ')
        .skip(1)
        .map(str::to_string)
        .collect();
    let args: Vec<&str> = fixed.iter().map(String::as_str).collect();
    let again = store.run(&args);
    assert_eq!(again["action"], "use");
    let ordered = store.run(&[
        "recall",
        "--about",
        "northwind",
        "--scope",
        "case-inv-1042",
        "--scope",
        "org",
    ]);
    assert_eq!(ordered["memories"][0]["scope"], "case-inv-1042");
    assert_eq!(ordered["memories"].as_array().unwrap().len(), 1);
}

#[test]
fn recall_narrow_over_the_limit() {
    let store = Store::new();
    for key in ["a", "b", "c"] {
        store.note(key, "org", &["northwind"]);
    }
    let out = store.run(&["recall", "--about", "northwind", "--limit", "2"]);
    insta::assert_json_snapshot!(out);
    let out = store.run(&["recall", "--about", "northwind", "--limit", "3"]);
    assert_eq!(out["action"], "use");
}

#[test]
fn recall_stale_with_fresh() {
    let store = Store::new();
    store.price();
    store.set_now("2026-12-20T18:00:00Z");
    let out = store.run(&["recall", "--about", "northwind", "--fresh", "30d"]);
    insta::assert_json_snapshot!(redact(&out));
    let out = store.run(&["recall", "--about", "northwind", "--fresh", "60days"]);
    assert_eq!(out["action"], "use");
}

#[test]
fn recall_by_key() {
    let store = Store::new();
    store.price();
    store.note("terms", "org", &["northwind"]);
    let out = store.run(&["recall", "--key", "terms", "--key", "widget-price"]);
    assert_eq!(out["memories"].as_array().unwrap().len(), 2);
    let out = store.run(&["recall", "--key", "terms", "--about", "discounts"]);
    assert_eq!(out["action"], "none");
}

#[test]
fn recall_text_output_for_every_action() {
    let store = Store::new();
    let none = store.text(&["recall", "--about", "northwind", "--output", "text"]);
    assert!(none.starts_with("Recall: none.\n"), "{none}");
    store.price();
    let used = store.text(&["recall", "--about", "northwind", "--output", "text"]);
    insta::assert_snapshot!(used);
    store.set_now("2026-12-20T18:00:00Z");
    let stale = store.text(&[
        "recall",
        "--about",
        "northwind",
        "--fresh",
        "30d",
        "--output",
        "text",
    ]);
    assert!(
        stale.starts_with("Recall: stale widget-price.\n"),
        "{stale}"
    );
    assert!(stale.contains("[stale] widget-price"));
    store.note("terms", "org", &["northwind"]);
    let narrow = store.text(&[
        "recall",
        "--about",
        "northwind",
        "--limit",
        "1",
        "--output",
        "text",
    ]);
    assert!(
        narrow.starts_with("Recall: narrow 2 keys matched."),
        "{narrow}"
    );
    store.note("terms", "case-7", &["northwind"]);
    let conflict = store.text(&["recall", "--about", "northwind", "--output", "text"]);
    assert!(
        conflict.starts_with("Recall: conflict on terms."),
        "{conflict}"
    );
}

#[test]
fn recall_text_errors_stay_json() {
    let store = Store::new();
    let err = store.fail(&["recall", "--about", "North", "--output", "text"], 4);
    assert_eq!(err["error"]["code"], "bad_slug");
}

#[test]
fn recall_refuses_bad_flags() {
    let store = Store::new();
    let err = store.fail(&["recall", "--scope", "org"], 2);
    assert_eq!(
        err["error"]["fix"],
        "relate-semantic recall --scope org --about SUBJ"
    );
    let err = store.fail(&["recall", "--about", "a", "--limit", "0"], 2);
    assert!(err["error"]["message"]
        .as_str()
        .unwrap()
        .contains("--limit"));
    assert_eq!(err["error"]["fix"], "relate-semantic recall --about a");
    let err = store.fail(&["recall", "--about", "a", "--limit", "21"], 2);
    assert_eq!(err["error"]["code"], "usage");
    let err = store.fail(&["recall", "--about", "a", "--fresh", "soon"], 2);
    assert!(err["error"]["message"]
        .as_str()
        .unwrap()
        .contains("--fresh"));
    assert_eq!(
        err["error"]["fix"],
        "relate-semantic recall --about a --fresh DURATION"
    );
    let err = store.fail(&["recall", "--about", "a", "--output", "yaml"], 2);
    assert_eq!(err["error"]["code"], "usage");
}
