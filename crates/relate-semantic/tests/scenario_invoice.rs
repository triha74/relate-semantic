//! The commands of examples/invoice, with the full JSON of every step.

mod common;

use common::*;

const CASE: &str = "case-inv-1042";
const PRICE: &str = "Under NW-2024, widgets are $100, with 10% off a widget line over $5000.";
const PRICE_SRC: &str = "Read from NW-2024 section 3 while checking INV-1042.";

fn seed(store: &Store) {
    store.run(&[
        "remember",
        "review-threshold",
        "--claim",
        "A person reviews any invoice whose billed total differs from the agreed total.",
        "--about",
        "invoices",
        "--about",
        "review",
        "--scope",
        "org",
        "--because",
        "Finance policy FP-7, effective 2026-01-01.",
    ]);
}

fn seed_price(t: &mut Transcript) -> serde_json::Value {
    t.run(&[
        "remember",
        "widget-price",
        "--claim",
        PRICE,
        "--about",
        "northwind",
        "--about",
        "nw-2024",
        "--about",
        "discounts",
        "--scope",
        "org",
        "--because",
        PRICE_SRC,
    ])
}

#[test]
fn run_discount_missing() {
    let store = Store::new();
    seed(&store);
    let mut t = Transcript::new(&store);
    let first = t.run(&[
        "recall",
        "--about",
        "northwind",
        "--about",
        "discounts",
        "--scope",
        CASE,
        "--scope",
        "org",
    ]);
    assert_eq!(first["action"], "none");
    seed_price(&mut t);
    t.run(&[
        "remember", "shortfall-handling", "--claim",
        "When a Northwind invoice omits the NW-2024 discount, pay the agreed total, not the billed total.",
        "--about", "northwind", "--about", "discounts", "--about", "shortfall", "--scope", "org",
        "--because", "REV-19 approved $9,000 for INV-1042 on 2026-11-10.",
    ]);
    store.advance("7d");
    let later = t.run(&[
        "recall",
        "--about",
        "northwind",
        "--about",
        "discounts",
        "--scope",
        "case-inv-1077",
        "--scope",
        "org",
    ]);
    assert_eq!(later["memories"].as_array().unwrap().len(), 2);
    assert_eq!(t.last_action(), "use");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_discount_rejected() {
    let store = Store::new();
    seed(&store);
    let mut t = Transcript::new(&store);
    t.run(&[
        "recall",
        "--about",
        "northwind",
        "--about",
        "discounts",
        "--scope",
        CASE,
        "--scope",
        "org",
    ]);
    seed_price(&mut t);
    t.run(&[
        "remember",
        "review-outcome",
        "--claim",
        "REV-21 rejected INV-1042. Do not pay it.",
        "--about",
        "inv-1042",
        "--scope",
        CASE,
        "--because",
        "human-review result rejected on REV-21.",
    ]);
    let before = t.run(&["recall", "--key", "review-outcome", "--scope", CASE]);
    assert_eq!(before["action"], "use");
    let pruned = t.run(&["prune", "--scope", CASE]);
    assert_eq!(pruned["generations"], 1);
    t.run(&["recall", "--key", "review-outcome", "--scope", CASE]);
    assert_eq!(t.last_action(), "none");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_discount_correct() {
    let store = Store::new();
    seed(&store);
    let mut t = Transcript::new(&store);
    let stored = seed_price(&mut t);
    t.run(&[
        "recall",
        "--about",
        "northwind",
        "--about",
        "nw-2024",
        "--scope",
        CASE,
        "--scope",
        "org",
    ]);
    store.advance("1h");
    let again = seed_price(&mut t);
    assert_eq!(
        stored, again,
        "repeating remember returns the stored generation"
    );
    t.run(&[
        "recall",
        "--key",
        "widget-price",
        "--scope",
        CASE,
        "--scope",
        "org",
    ]);
    assert_eq!(t.last_action(), "use");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_vendor_rejected() {
    let store = Store::new();
    seed(&store);
    let mut t = Transcript::new(&store);
    t.run(&[
        "recall",
        "--about",
        "northwind",
        "--about",
        "vendor-list",
        "--scope",
        CASE,
        "--scope",
        "org",
    ]);
    t.run(&[
        "remember",
        "vendor-status",
        "--claim",
        "Northwind is not on the approved vendor list.",
        "--about",
        "northwind",
        "--about",
        "vendor-list",
        "--scope",
        "org",
        "--because",
        "Checked the approved vendor list for INV-1042 on 2026-11-10.",
    ]);
    store.advance("2h");
    t.run(&[
        "correct", "vendor-status", "--scope", "org", "--agent", "human", "--claim",
        "Northwind is not on the approved vendor list. A person approves each Northwind invoice before payment.",
        "--because", "REV-22 approved paying INV-1042 anyway.",
    ]);
    let log = t.run(&["log", "vendor-status", "--json"]);
    assert_eq!(log["generations"][0]["agent"], "human");
    assert_eq!(log["generations"][1]["live"], false);
    store.advance("3d");
    let out = t.run(&["recall", "--about", "northwind", "--about", "vendor-list"]);
    assert_eq!(out["memories"][0]["generation"], 2);
    assert_eq!(t.last_action(), "use");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_missing_agreement() {
    let store = Store::new();
    seed(&store);
    let mut t = Transcript::new(&store);
    t.run(&["recall", "--about", "northwind", "--about", "nw-2024"]);
    t.run(&[
        "remember",
        "widget-price",
        "--claim",
        "For INV-1042 only, finance quoted $100 per widget by phone while NW-2024 was missing.",
        "--about",
        "northwind",
        "--about",
        "nw-2024",
        "--scope",
        CASE,
        "--because",
        "Phone call with finance on 2026-11-10. NW-2024 was not on file.",
    ]);
    store.advance("1h");
    let taken = t.refused(
        "key_taken",
        &[
            "remember",
            "widget-price",
            "--claim",
            PRICE,
            "--about",
            "northwind",
            "--about",
            "nw-2024",
            "--about",
            "discounts",
            "--scope",
            CASE,
            "--because",
            PRICE_SRC,
        ],
    );
    assert_eq!(
        taken["error"]["fix"],
        "relate-semantic correct widget-price --scope case-inv-1042 --claim TEXT --because TEXT --about discounts --about northwind --about nw-2024"
    );
    seed_price(&mut t);
    let out = t.run(&["recall", "--about", "northwind", "--about", "nw-2024"]);
    assert_eq!(out["scopes"], serde_json::json!([CASE, "org"]));
    assert_eq!(t.last_action(), "conflict");
    insta::assert_json_snapshot!(t.steps());
}
