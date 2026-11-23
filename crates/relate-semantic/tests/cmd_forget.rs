mod common;

use common::*;
use serde_json::json;

#[test]
fn forget_writes_a_tombstone_and_repeats_as_not_forgotten() {
    let store = Store::new();
    store.price();
    store.set_now("2026-11-17T17:00:00Z");
    let out = store.run(&[
        "forget",
        "widget-price",
        "--scope",
        "org",
        "--because",
        "NW-2024 was withdrawn.",
    ]);
    insta::assert_json_snapshot!(out);
    let again = store.run(&[
        "forget",
        "widget-price",
        "--scope",
        "org",
        "--because",
        "NW-2024 was withdrawn.",
    ]);
    assert_eq!(
        again,
        json!({"key": "widget-price", "scope": "org", "forgotten": false})
    );
    let recalled = store.run(&["recall", "--key", "widget-price"]);
    assert_eq!(recalled["action"], "none");
}

#[test]
fn a_forgotten_key_can_be_remembered_again_as_the_next_generation() {
    let store = Store::new();
    store.price();
    store.run(&[
        "forget",
        "widget-price",
        "--scope",
        "org",
        "--because",
        "withdrawn",
    ]);
    let back = store.price();
    assert_eq!(back["generation"], 3);
}

#[test]
fn forget_validates_and_requires_its_flags() {
    let store = Store::new();
    let err = store.fail(
        &["forget", "widget-price", "--scope", "org", "--because", ""],
        4,
    );
    assert_eq!(err["error"]["code"], "empty_field");
    let err = store.fail(&["forget", "widget-price", "--scope", "org"], 2);
    assert_eq!(
        err["error"]["fix"],
        "relate-semantic forget KEY --scope SCOPE --because TEXT [--agent NAME]"
    );
    store.set_agent(None);
    let err = store.fail(
        &["forget", "widget-price", "--scope", "org", "--because", "x"],
        2,
    );
    assert_eq!(err["error"]["message"], "forget requires --agent");
}
