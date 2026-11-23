mod support;

use support::*;

#[test]
fn remember_recall_show_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db");
    let memory = open_at(path.to_str().unwrap());
    let written = memory
        .remember(remember(
            "widget-price",
            "org",
            PRICE,
            &["northwind", "nw-2024", "discounts"],
        ))
        .unwrap();
    assert_eq!(written["generation"], 1);
    assert_eq!(written["at"], T0);

    let recalled = memory
        .recall(recall(&["northwind", "discounts"], &["org"]))
        .unwrap();
    assert_eq!(recalled["action"], "use");
    assert_eq!(recalled["memories"][0]["claim"], PRICE);
    assert_eq!(
        recalled["memories"][0]["about"],
        serde_json::json!(["discounts", "northwind", "nw-2024"])
    );

    drop(memory);
    let reopened = open_at(path.to_str().unwrap());
    let shown = reopened.show("widget-price").unwrap();
    assert_eq!(shown["memories"][0]["scope"], "org");
    let pruned = reopened.prune("org").unwrap();
    assert_eq!(pruned["generations"], 1);
    assert_eq!(reopened.scopes().unwrap()["scopes"], serde_json::json!([]));
}
