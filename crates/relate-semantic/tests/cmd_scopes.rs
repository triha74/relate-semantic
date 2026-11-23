mod common;

use common::*;
use serde_json::json;

#[test]
fn no_verb_on_a_missing_store_lists_nothing_and_creates_nothing() {
    let store = Store::new();
    assert_eq!(store.run(&[]), json!({"scopes": []}));
    assert!(!store.exists());
}

#[test]
fn no_verb_lists_scopes_with_counts() {
    let store = Store::new();
    store.price();
    store.run(&[
        "correct",
        "widget-price",
        "--scope",
        "org",
        "--claim",
        PRICE_2025,
        "--because",
        "NW-2025 signed.",
    ]);
    store.note("widget-price", "case-inv-1042", &["northwind"]);
    store.run(&[
        "forget",
        "widget-price",
        "--scope",
        "case-inv-1042",
        "--because",
        "Case closed.",
    ]);
    let out = store.run(&[]);
    insta::assert_json_snapshot!(out);
}

#[test]
fn a_foreign_store_is_refused() {
    let store = Store::new();
    {
        let db = relate::Database::open(&store.path()).unwrap();
        db.connect()
            .unwrap()
            .query("CREATE (:Widget {name: 'x'})")
            .unwrap();
    }
    let err = store.fail(&[], 4);
    assert_eq!(err["error"]["code"], "foreign_store");
}
