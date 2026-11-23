//! The invoice scenario through the library, with a clock the test moves.
//! The CLI scenario test checks the same story through the binary.

mod support;

use relate_semantic_core::Recall;
use serde_json::json;
use support::*;

const CASE: &str = "case-inv-1042";

fn price_recall(scopes: &[&str]) -> Recall {
    recall(&["northwind", "discounts"], scopes)
}

#[test]
fn the_first_case_teaches_the_second() {
    let (memory, clock) = open_manual();
    assert_eq!(
        memory.recall(price_recall(&[CASE, "org"])).unwrap()["action"],
        "none"
    );
    memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE,
            &["northwind", "nw-2024", "discounts"],
        ))
        .unwrap();
    clock.advance(days(7));
    let out = memory
        .recall(price_recall(&["case-inv-1077", "org"]))
        .unwrap();
    assert_eq!(out["action"], "use");
    assert_eq!(out["memories"][0]["claim"], PRICE);
    assert_eq!(out["memories"][0]["at"], T0);
}

#[test]
fn a_case_override_wins_only_where_the_case_scope_comes_first() {
    let memory = open();
    memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE,
            &["northwind", "discounts"],
        ))
        .unwrap();
    memory
        .remember(remember(
            "widget-price",
            CASE,
            "INV-1042 pays $90 per widget.",
            &["northwind", "discounts"],
        ))
        .unwrap();
    let conflict = memory.recall(price_recall(&[])).unwrap();
    assert_eq!(
        conflict,
        json!({
            "action": "conflict",
            "key": "widget-price",
            "scopes": [CASE, "org"],
            "fix": "relate-semantic recall --key widget-price --scope case-inv-1042 --scope org",
        })
    );
    let case_first = memory.recall(price_recall(&[CASE, "org"])).unwrap();
    assert_eq!(case_first["memories"][0]["scope"], CASE);
    let other_case = memory
        .recall(price_recall(&["case-inv-1077", "org"]))
        .unwrap();
    assert_eq!(other_case["memories"][0]["scope"], "org");
}

#[test]
fn a_new_agreement_is_a_correction_and_the_old_price_stays_in_the_log() {
    let (memory, clock) = open_manual();
    memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE,
            &["northwind", "nw-2024", "discounts"],
        ))
        .unwrap();
    clock.advance(days(21));
    let taken = memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE_2025,
            &["northwind", "nw-2025", "discounts"],
        ))
        .unwrap_err();
    assert_eq!(taken.code, "key_taken");
    memory
        .correct(correct(
            "widget-price",
            "org",
            PRICE_2025,
            Some(&["northwind", "nw-2025", "discounts"]),
        ))
        .unwrap();
    assert_eq!(
        memory.recall(recall(&["nw-2024"], &[])).unwrap()["action"],
        "none"
    );
    let log = memory.log("widget-price", Some("org")).unwrap();
    assert_eq!(log["generations"][1]["claim"], PRICE);
    assert_eq!(log["generations"][0]["claim"], PRICE_2025);
}

#[test]
fn closing_a_case_prunes_its_notes_and_keeps_org_facts() {
    let memory = open();
    memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE,
            &["northwind", "discounts"],
        ))
        .unwrap();
    memory
        .remember(remember(
            "review-outcome",
            CASE,
            "REV-21 rejected INV-1042.",
            &["inv-1042"],
        ))
        .unwrap();
    memory.prune(CASE).unwrap();
    assert_eq!(
        memory
            .recall(recall_keys(&["review-outcome"], &[CASE]))
            .unwrap()["action"],
        "none"
    );
    assert_eq!(memory.recall(price_recall(&[])).unwrap()["action"], "use");
}
