mod common;

use common::*;
use serde_json::json;

#[test]
fn prune_deletes_a_scope_and_reports_counts() {
    let store = Store::new();
    store.price();
    store.note("widget-price", "case-inv-1042", &["northwind"]);
    store.note("terms", "case-inv-1042", &["northwind"]);
    let out = store.run(&["prune", "--scope", "case-inv-1042"]);
    insta::assert_json_snapshot!(out);
    let recalled = store.run(&["recall", "--about", "northwind"]);
    assert_eq!(recalled["action"], "use");
    assert_eq!(recalled["memories"].as_array().unwrap().len(), 1);
    let again = store.run(&["prune", "--scope", "case-inv-1042"]);
    assert_eq!(
        again,
        json!({"pruned": "case-inv-1042", "keys": 0, "generations": 0})
    );
}

#[test]
fn prune_on_a_missing_store_creates_nothing() {
    let store = Store::new();
    let out = store.run(&["prune", "--scope", "org"]);
    assert_eq!(out["generations"], 0);
    assert!(!store.exists());
}

#[test]
fn prune_validates_and_requires_the_scope() {
    let store = Store::new();
    let err = store.fail(&["prune", "--scope", "Case 7"], 4);
    assert_eq!(err["error"]["fix"], "relate-semantic prune --scope case-7");
    let err = store.fail(&["prune"], 2);
    assert_eq!(err["error"]["fix"], "relate-semantic prune --scope SCOPE");
}
