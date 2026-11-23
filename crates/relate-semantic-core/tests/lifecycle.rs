mod support;

use serde_json::json;
use support::*;

#[test]
fn remember_correct_forget_and_remember_again() {
    let (memory, clock) = open_manual();
    let one = memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE,
            &["northwind", "nw-2024"],
        ))
        .unwrap();
    assert_eq!(
        one,
        json!({"key": "widget-price", "scope": "org", "generation": 1, "at": T0})
    );

    clock.advance(days(7));
    let two = memory
        .correct(correct(
            "widget-price",
            "org",
            PRICE_2025,
            Some(&["northwind", "nw-2025"]),
        ))
        .unwrap();
    assert_eq!(two["generation"], 2);
    assert_eq!(two["at"], "2026-11-17T18:00:00Z");

    clock.advance(days(1));
    let three = memory.forget(forget("widget-price", "org")).unwrap();
    assert_eq!(three["generation"], 3);
    assert_eq!(three["forgotten"], true);

    let none = memory.recall(recall_keys(&["widget-price"], &[])).unwrap();
    assert_eq!(none["action"], "none");

    clock.advance(days(1));
    let four = memory
        .remember(remember("widget-price", "org", PRICE_2025, &["northwind"]))
        .unwrap();
    assert_eq!(four["generation"], 4);

    let log = memory.log("widget-price", None).unwrap();
    let gens: Vec<u64> = log["generations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["generation"].as_u64().unwrap())
        .collect();
    assert_eq!(gens, vec![4, 3, 2, 1]);
    let live: Vec<bool> = log["generations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["live"].as_bool().unwrap())
        .collect();
    assert_eq!(live, vec![true, false, false, false]);
    assert_eq!(log["generations"][1]["claim"], json!(null));
    assert_eq!(log["generations"][1]["about"], json!([]));
}

#[test]
fn a_repeated_remember_returns_the_original_and_writes_nothing() {
    let (memory, clock) = open_manual();
    let first = memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE,
            &["nw-2024", "northwind"],
        ))
        .unwrap();
    clock.advance(days(3));
    let again = memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE,
            &["northwind", "nw-2024"],
        ))
        .unwrap();
    assert_eq!(first, again);
    let log = memory.log("widget-price", None).unwrap();
    assert_eq!(log["generations"].as_array().unwrap().len(), 1);
}

#[test]
fn a_different_remember_is_key_taken_and_names_correct() {
    let memory = open();
    memory
        .remember(remember("widget-price", "org", PRICE, &["northwind"]))
        .unwrap();
    let err = memory
        .remember(remember("widget-price", "org", PRICE_2025, &["northwind"]))
        .unwrap_err();
    assert_eq!(err.code, "key_taken");
    assert_eq!(err.exit_code(), 4);
    assert_eq!(
        err.fix,
        "relate-semantic correct widget-price --scope org --claim TEXT --because TEXT"
    );
    assert!(err.message.contains(PRICE));

    let mut changed_source = remember("widget-price", "org", PRICE, &["northwind"]);
    changed_source.because = "another source".into();
    let err = memory.remember(changed_source).unwrap_err();
    assert_eq!(err.code, "key_taken");

    let err = memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE,
            &["northwind", "discounts"],
        ))
        .unwrap_err();
    assert_eq!(
        err.fix,
        "relate-semantic correct widget-price --scope org --claim TEXT --because TEXT --about discounts --about northwind"
    );
}

#[test]
fn the_same_key_in_another_scope_is_a_separate_memory() {
    let memory = open();
    memory
        .remember(remember("widget-price", "org", PRICE, &["northwind"]))
        .unwrap();
    let case = memory
        .remember(remember(
            "widget-price",
            "inv-1077",
            "For INV-1077 only, widgets are $95.",
            &["northwind"],
        ))
        .unwrap();
    assert_eq!(case["generation"], 1);
    let shown = memory.show("widget-price").unwrap();
    let scopes: Vec<&str> = shown["memories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["scope"].as_str().unwrap())
        .collect();
    assert_eq!(scopes, vec!["inv-1077", "org"]);
}

#[test]
fn forget_of_a_key_that_is_not_live_writes_nothing() {
    let memory = open();
    let out = memory.forget(forget("widget-price", "org")).unwrap();
    assert_eq!(
        out,
        json!({"key": "widget-price", "scope": "org", "forgotten": false})
    );
    memory
        .remember(remember("widget-price", "org", PRICE, &["northwind"]))
        .unwrap();
    memory.forget(forget("widget-price", "org")).unwrap();
    let again = memory.forget(forget("widget-price", "org")).unwrap();
    assert_eq!(again["forgotten"], false);
    let log = memory.log("widget-price", Some("org")).unwrap();
    assert_eq!(log["generations"].as_array().unwrap().len(), 2);
}

#[test]
fn scopes_count_live_keys_and_generations() {
    let memory = open();
    memory.remember(remember("a", "org", "a", &["s"])).unwrap();
    memory.remember(remember("b", "org", "b", &["s"])).unwrap();
    memory.correct(correct("b", "org", "b2", None)).unwrap();
    memory
        .remember(remember("c", "inv-1042", "c", &["s"]))
        .unwrap();
    memory.forget(forget("c", "inv-1042")).unwrap();
    assert_eq!(
        memory.scopes().unwrap(),
        json!({"scopes": [
            {"name": "inv-1042", "live": 0, "generations": 2},
            {"name": "org", "live": 2, "generations": 3}
        ]})
    );
}
