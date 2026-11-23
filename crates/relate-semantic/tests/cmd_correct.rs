mod common;

use common::*;
use serde_json::json;

#[test]
fn correct_writes_the_next_generation() {
    let store = Store::new();
    store.price();
    store.set_now("2026-11-17T17:00:00Z");
    let out = store.run(&[
        "correct",
        "widget-price",
        "--scope",
        "org",
        "--claim",
        PRICE_2025,
        "--because",
        "NW-2025 replaced NW-2024 on 2026-11-01.",
        "--about",
        "northwind",
        "--about",
        "nw-2025",
        "--about",
        "discounts",
    ]);
    insta::assert_json_snapshot!(out);
    let shown = store.run(&["show", "widget-price"]);
    assert_eq!(
        shown["memories"][0]["about"],
        json!(["discounts", "northwind", "nw-2025"])
    );
    assert_eq!(shown["memories"][0]["generation"], 2);
}

#[test]
fn correct_keeps_subjects_when_about_is_omitted_and_repeats_cleanly() {
    let store = Store::new();
    store.price();
    let args = [
        "correct",
        "widget-price",
        "--scope",
        "org",
        "--claim",
        PRICE_2025,
        "--because",
        "new agreement",
    ];
    let first = store.run(&args);
    store.set_now("2026-11-12T00:00:00Z");
    let again = store.run(&args);
    assert_eq!(first, again);
    let shown = store.run(&["show", "widget-price"]);
    assert_eq!(
        shown["memories"][0]["about"],
        json!(["discounts", "northwind", "nw-2024"])
    );
}

#[test]
fn correct_on_a_missing_key_is_no_memory() {
    let store = Store::new();
    let err = store.fail(
        &[
            "correct",
            "widget-price",
            "--scope",
            "org",
            "--claim",
            PRICE,
            "--because",
            PRICE_SRC,
        ],
        4,
    );
    insta::assert_json_snapshot!(err);
}

#[test]
fn correct_validates_and_requires_its_flags() {
    let store = Store::new();
    let err = store.fail(
        &[
            "correct",
            "widget-price",
            "--scope",
            "Org",
            "--claim",
            " ",
            "--because",
            "x",
        ],
        4,
    );
    assert_eq!(err["error"]["code"], "bad_slug");
    assert_eq!(err["error"]["errors"][1]["code"], "empty_field");
    let err = store.fail(
        &["correct", "widget-price", "--claim", "c", "--because", "x"],
        2,
    );
    assert_eq!(
        err["error"]["fix"],
        "relate-semantic correct KEY --scope SCOPE --claim TEXT --because TEXT [--about SUBJ]... [--agent NAME]"
    );
    store.set_agent(None);
    let err = store.fail(
        &[
            "correct",
            "widget-price",
            "--scope",
            "org",
            "--claim",
            "c",
            "--because",
            "x",
        ],
        2,
    );
    assert_eq!(err["error"]["message"], "correct requires --agent");
}
