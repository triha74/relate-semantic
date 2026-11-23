mod common;

use common::*;

fn history(store: &Store) {
    store.price();
    store.set_now("2026-11-12T09:00:00Z");
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
    store.set_now("2026-11-13T09:00:00Z");
    store.note("widget-price", "case-inv-1042", &["northwind"]);
    store.set_now("2026-11-14T09:00:00Z");
    store.set_agent(Some("finance"));
    store.run(&[
        "forget",
        "widget-price",
        "--scope",
        "org",
        "--because",
        "Under renegotiation.",
    ]);
}

#[test]
fn log_lists_every_generation_newest_first_as_json_when_piped() {
    let store = Store::new();
    history(&store);
    let out = store.run(&["log", "widget-price"]);
    insta::assert_json_snapshot!(out);
}

#[test]
fn log_filters_by_scope_and_json_forces_json() {
    let store = Store::new();
    history(&store);
    let out = store.run(&["log", "widget-price", "--scope", "org", "--json"]);
    let generations: Vec<u64> = out["generations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["generation"].as_u64().unwrap())
        .collect();
    assert_eq!(generations, vec![3, 2, 1]);
    assert_eq!(out["generations"][0]["agent"], "finance");
    assert_eq!(out["generations"][0]["claim"], serde_json::Value::Null);
}

#[test]
fn log_on_a_missing_key_is_no_memory() {
    let store = Store::new();
    let err = store.fail(&["log", "widget-price", "--scope", "org"], 4);
    assert_eq!(
        err["error"]["message"],
        "widget-price has no generations in org"
    );
    assert_eq!(
        err["error"]["fix"],
        "relate-semantic remember widget-price --claim TEXT --about SUBJ --scope org --because TEXT"
    );
}

#[test]
fn log_validates_its_slugs() {
    let store = Store::new();
    let err = store.fail(&["log", "Widget", "--scope", "Org"], 4);
    assert_eq!(
        err["error"]["fix"],
        "relate-semantic log widget --scope org"
    );
    assert_eq!(err["error"]["errors"].as_array().unwrap().len(), 2);
}
