//! One key through its whole life, ending in every recall action at least
//! once: remember, recall, correct, stale, forget, remember again, conflict,
//! narrow, and prune.

mod common;

use common::*;
use serde_json::json;

#[test]
fn a_key_through_its_whole_life() {
    let store = Store::new();
    let mut t = Transcript::new(&store);
    t.run(&["recall", "--about", "northwind"]);
    t.run(&[
        "remember",
        "widget-price",
        "--claim",
        PRICE,
        "--about",
        "northwind",
        "--about",
        "nw-2024",
        "--scope",
        "org",
        "--because",
        PRICE_SRC,
    ]);
    t.run(&["recall", "--about", "northwind"]);
    store.advance("40d");
    t.run(&["recall", "--about", "northwind", "--fresh", "30d"]);
    t.run(&[
        "correct",
        "widget-price",
        "--scope",
        "org",
        "--claim",
        PRICE_2025,
        "--about",
        "northwind",
        "--about",
        "nw-2025",
        "--because",
        "NW-2025 replaced NW-2024 on 2026-12-01.",
    ]);
    t.run(&["recall", "--about", "northwind", "--fresh", "30d"]);
    t.run(&["recall", "--about", "nw-2024"]);
    t.run(&[
        "forget",
        "widget-price",
        "--scope",
        "org",
        "--because",
        "Prices are under renegotiation.",
    ]);
    t.refused("no_memory", &["show", "widget-price"]);
    store.advance("1d");
    t.run(&[
        "remember",
        "widget-price",
        "--claim",
        PRICE_2025,
        "--about",
        "northwind",
        "--scope",
        "org",
        "--because",
        "Renegotiation ended with NW-2025 unchanged.",
    ]);
    t.run(&[
        "remember",
        "widget-price",
        "--claim",
        "Case 7 pays $90 per widget.",
        "--about",
        "northwind",
        "--scope",
        "case-7",
        "--because",
        "Approved by finance for case 7.",
    ]);
    t.run(&["recall", "--about", "northwind"]);
    t.run(&[
        "recall",
        "--about",
        "northwind",
        "--scope",
        "case-7",
        "--scope",
        "org",
    ]);
    t.run(&[
        "remember",
        "payment-terms",
        "--claim",
        "Northwind pays net 30.",
        "--about",
        "northwind",
        "--scope",
        "org",
        "--because",
        "NW-2025 section 5.",
    ]);
    t.run(&[
        "recall",
        "--about",
        "northwind",
        "--scope",
        "org",
        "--scope",
        "case-7",
        "--limit",
        "1",
    ]);
    t.run(&["prune", "--scope", "case-7"]);
    t.run(&["log", "widget-price"]);
    t.run(&[]);

    let actions: Vec<String> = t
        .steps()
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["out"]["action"].as_str().map(str::to_string))
        .collect();
    assert_eq!(
        actions,
        vec!["none", "use", "stale", "use", "none", "conflict", "use", "narrow"]
    );
    let listing = store.run(&[]);
    assert_eq!(
        listing,
        json!({"scopes": [{"name": "org", "live": 2, "generations": 5}]})
    );
    insta::assert_json_snapshot!(t.steps());
}
