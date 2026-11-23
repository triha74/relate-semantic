//! Scopes through the library: isolation, ordering, and the listing.

mod support;

use serde_json::json;
use support::*;

#[test]
fn the_same_key_lives_independently_in_two_scopes() {
    let memory = open();
    memory
        .remember(remember("price", "org", PRICE, &["nw"]))
        .unwrap();
    memory
        .remember(remember("price", "case-7", "Case 7 pays $90.", &["nw"]))
        .unwrap();
    memory
        .correct(correct("price", "org", PRICE_2025, None))
        .unwrap();

    let case = memory.recall(recall(&["nw"], &["case-7"])).unwrap();
    assert_eq!(case["memories"][0]["claim"], "Case 7 pays $90.");
    assert_eq!(case["memories"][0]["generation"], 1);

    let both = memory.recall(recall(&["nw"], &[])).unwrap();
    assert_eq!(both["action"], "conflict");
    assert_eq!(both["scopes"], json!(["case-7", "org"]));

    let ordered = memory.recall(recall(&["nw"], &["case-7", "org"])).unwrap();
    assert_eq!(ordered["action"], "use");
    assert_eq!(ordered["memories"].as_array().unwrap().len(), 1);
    assert_eq!(ordered["memories"][0]["scope"], "case-7");
}

#[test]
fn forgetting_in_one_scope_leaves_the_other() {
    let memory = open();
    memory
        .remember(remember("price", "org", PRICE, &["nw"]))
        .unwrap();
    memory
        .remember(remember("price", "case-7", "Case 7 pays $90.", &["nw"]))
        .unwrap();
    memory.forget(forget("price", "case-7")).unwrap();
    let out = memory.recall(recall(&["nw"], &[])).unwrap();
    assert_eq!(out["action"], "use");
    assert_eq!(out["memories"][0]["scope"], "org");
}

#[test]
fn an_unknown_scope_recalls_none() {
    let memory = open();
    memory
        .remember(remember("price", "org", PRICE, &["nw"]))
        .unwrap();
    let out = memory.recall(recall(&["nw"], &["nobody"])).unwrap();
    assert_eq!(out["action"], "none");
}

#[test]
fn the_listing_counts_live_keys_and_generations_per_scope() {
    let memory = open();
    assert_eq!(memory.scopes().unwrap(), json!({"scopes": []}));
    memory
        .remember(remember("price", "org", PRICE, &["nw"]))
        .unwrap();
    memory
        .correct(correct("price", "org", PRICE_2025, None))
        .unwrap();
    memory
        .remember(remember("terms", "org", "Net 30.", &["nw"]))
        .unwrap();
    memory
        .remember(remember("price", "case-7", "Case 7 pays $90.", &["nw"]))
        .unwrap();
    memory.forget(forget("price", "case-7")).unwrap();
    assert_eq!(
        memory.scopes().unwrap(),
        json!({"scopes": [
            {"name": "case-7", "live": 0, "generations": 2},
            {"name": "org", "live": 2, "generations": 3},
        ]})
    );
}

#[test]
fn fresh_compares_against_the_clock() {
    let (memory, clock) = open_manual();
    memory
        .remember(remember("price", "org", PRICE, &["nw"]))
        .unwrap();
    let query = || relate_semantic_core::Recall {
        fresh: fresh(7),
        ..recall(&["nw"], &[])
    };
    assert_eq!(memory.recall(query()).unwrap()["action"], "use");
    clock.advance(days(8));
    let out = memory.recall(query()).unwrap();
    assert_eq!(out["action"], "stale");
    assert_eq!(out["stale"], json!(["price"]));
    memory
        .correct(correct("price", "org", PRICE_2025, None))
        .unwrap();
    assert_eq!(memory.recall(query()).unwrap()["action"], "use");
}
