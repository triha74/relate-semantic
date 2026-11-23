mod support;

use serde_json::json;
use support::*;

#[test]
fn correct_carries_the_subjects_when_about_is_omitted() {
    let memory = open();
    memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE,
            &["northwind", "nw-2024"],
        ))
        .unwrap();
    memory
        .correct(correct("widget-price", "org", PRICE_2025, None))
        .unwrap();
    let shown = memory.show("widget-price").unwrap();
    assert_eq!(
        shown["memories"][0]["about"],
        json!(["northwind", "nw-2024"])
    );
    assert_eq!(shown["memories"][0]["claim"], PRICE_2025);
    assert_eq!(shown["memories"][0]["generation"], 2);
}

#[test]
fn correct_with_about_replaces_the_whole_list() {
    let memory = open();
    memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE,
            &["northwind", "nw-2024"],
        ))
        .unwrap();
    memory
        .correct(correct(
            "widget-price",
            "org",
            PRICE_2025,
            Some(&["nw-2025", "northwind"]),
        ))
        .unwrap();
    let old = memory.recall(recall(&["nw-2024"], &[])).unwrap();
    assert_eq!(old["action"], "none");
    let new = memory.recall(recall(&["nw-2025"], &[])).unwrap();
    assert_eq!(new["memories"][0]["about"], json!(["northwind", "nw-2025"]));
}

#[test]
fn an_empty_about_list_on_correct_keeps_the_subjects() {
    let memory = open();
    memory
        .remember(remember("widget-price", "org", PRICE, &["northwind"]))
        .unwrap();
    memory
        .correct(correct("widget-price", "org", PRICE_2025, Some(&[])))
        .unwrap();
    assert_eq!(
        memory.show("widget-price").unwrap()["memories"][0]["about"],
        json!(["northwind"])
    );
}

#[test]
fn a_correction_that_changes_nothing_writes_nothing() {
    let (memory, clock) = open_manual();
    memory
        .remember(remember("widget-price", "org", PRICE_2025, &["northwind"]))
        .unwrap();
    let first = memory
        .correct(correct("widget-price", "org", PRICE, None))
        .unwrap();
    clock.advance(days(1));
    let out = memory
        .correct(correct("widget-price", "org", PRICE, None))
        .unwrap();
    assert_eq!(out, first);
    assert_eq!(
        memory.log("widget-price", None).unwrap()["generations"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn a_new_source_for_the_same_claim_confirms_a_stale_claim() {
    let (memory, clock) = open_manual();
    memory
        .remember(remember("widget-price", "org", PRICE, &["northwind"]))
        .unwrap();
    clock.advance(days(40));
    let query = || relate_semantic_core::Recall {
        fresh: fresh(30),
        ..recall(&["northwind"], &[])
    };
    assert_eq!(memory.recall(query()).unwrap()["action"], "stale");
    let mut confirm = correct("widget-price", "org", PRICE, None);
    confirm.because = "Checked NW-2024 again on 2026-12-20. Unchanged.".into();
    let out = memory.correct(confirm).unwrap();
    assert_eq!(out["generation"], 2);
    let fresh_now = memory.recall(query()).unwrap();
    assert_eq!(fresh_now["action"], "use");
    assert_eq!(fresh_now["memories"][0]["claim"], PRICE);
}

#[test]
fn a_correction_that_only_changes_subjects_is_a_new_generation() {
    let memory = open();
    memory
        .remember(remember("widget-price", "org", PRICE, &["northwind"]))
        .unwrap();
    let out = memory
        .correct(correct(
            "widget-price",
            "org",
            PRICE,
            Some(&["northwind", "discounts"]),
        ))
        .unwrap();
    assert_eq!(out["generation"], 2);
}

#[test]
fn correct_on_a_missing_key_is_no_memory_and_names_remember() {
    let memory = open();
    let err = memory
        .correct(correct("widget-price", "org", PRICE, None))
        .unwrap_err();
    assert_eq!(err.code, "no_memory");
    assert_eq!(err.message, "widget-price is not live in org");
    assert_eq!(
        err.fix,
        "relate-semantic remember widget-price --claim TEXT --about SUBJ --scope org --because TEXT"
    );
    let err = memory
        .correct(correct(
            "widget-price",
            "org",
            PRICE,
            Some(&["nw-2024", "northwind"]),
        ))
        .unwrap_err();
    assert_eq!(
        err.fix,
        "relate-semantic remember widget-price --claim TEXT --about northwind --about nw-2024 --scope org --because TEXT"
    );
}

#[test]
fn correct_on_a_forgotten_key_is_no_memory_and_keeps_its_old_subjects() {
    let memory = open();
    memory
        .remember(remember("widget-price", "org", PRICE, &["northwind"]))
        .unwrap();
    memory.forget(forget("widget-price", "org")).unwrap();
    let err = memory
        .correct(correct("widget-price", "org", PRICE, None))
        .unwrap_err();
    assert_eq!(err.code, "no_memory");
    assert_eq!(err.message, "widget-price was forgotten in org");
    assert_eq!(
        err.fix,
        "relate-semantic remember widget-price --claim TEXT --about SUBJ --scope org --because TEXT"
    );
}

#[test]
fn the_tombstone_is_not_live_and_the_previous_generation_is_retired() {
    let memory = open();
    memory
        .remember(remember("widget-price", "org", PRICE, &["northwind"]))
        .unwrap();
    memory
        .correct(correct("widget-price", "org", PRICE_2025, None))
        .unwrap();
    memory.forget(forget("widget-price", "org")).unwrap();
    let log = memory.log("widget-price", Some("org")).unwrap();
    for row in log["generations"].as_array().unwrap() {
        assert_eq!(row["live"], false);
    }
    assert_eq!(log["generations"][0]["forgotten"], true);
    assert_eq!(
        log["generations"][0]["because"],
        "The case override no longer applies."
    );
}

#[test]
fn log_is_capped_and_newest_first() {
    let (memory, clock) = open_manual();
    memory.remember(remember("k", "org", "c0", &["s"])).unwrap();
    for n in 1..60 {
        clock.advance(std::time::Duration::from_secs(60));
        memory
            .correct(correct("k", "org", &format!("c{n}"), None))
            .unwrap();
    }
    let log = memory.log("k", None).unwrap();
    let rows = log["generations"].as_array().unwrap();
    assert_eq!(rows.len(), relate_semantic_core::LOG_CAP);
    assert_eq!(rows[0]["generation"], 60);
    assert_eq!(rows[49]["generation"], 11);
}
