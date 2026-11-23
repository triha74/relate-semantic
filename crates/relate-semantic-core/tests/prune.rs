mod support;

use serde_json::json;
use support::*;

#[test]
fn prune_removes_one_scope_and_its_history() {
    let memory = open();
    memory
        .remember(remember("price", "org", PRICE, &["nw", "discounts"]))
        .unwrap();
    memory
        .remember(remember(
            "price",
            "case-7",
            "Case 7 pays $90.",
            &["nw", "case-7"],
        ))
        .unwrap();
    memory
        .correct(correct("price", "case-7", "Case 7 pays $95.", None))
        .unwrap();
    memory
        .remember(remember("terms", "case-7", "Net 10.", &["nw"]))
        .unwrap();

    let out = memory.prune("case-7").unwrap();
    assert_eq!(
        out,
        json!({"pruned": "case-7", "keys": 2, "generations": 3})
    );

    assert_eq!(
        memory.scopes().unwrap(),
        json!({"scopes": [{"name": "org", "live": 1, "generations": 1}]})
    );
    let log = memory.log("price", None).unwrap();
    assert_eq!(log["generations"].as_array().unwrap().len(), 1);
    assert_eq!(memory.log("terms", None).unwrap_err().code, "no_memory");
    assert_eq!(
        memory.recall(recall(&["nw"], &[])).unwrap()["memories"][0]["scope"],
        "org"
    );
}

#[test]
fn prune_of_an_empty_scope_reports_zero_and_repeats() {
    let memory = open();
    let out = memory.prune("nobody").unwrap();
    assert_eq!(
        out,
        json!({"pruned": "nobody", "keys": 0, "generations": 0})
    );
    memory.remember(remember("k", "org", "c", &["a"])).unwrap();
    memory.prune("org").unwrap();
    assert_eq!(
        memory.prune("org").unwrap(),
        json!({"pruned": "org", "keys": 0, "generations": 0})
    );
}

#[test]
fn a_pruned_scope_starts_again_at_generation_one() {
    let memory = open();
    memory
        .remember(remember("k", "case-7", "c", &["a"]))
        .unwrap();
    memory.correct(correct("k", "case-7", "d", None)).unwrap();
    memory.prune("case-7").unwrap();
    let again = memory
        .remember(remember("k", "case-7", "e", &["a"]))
        .unwrap();
    assert_eq!(again["generation"], 1);
}

#[test]
fn prune_keeps_subjects_other_scopes_still_use() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db");
    {
        let memory = open_at(path.to_str().unwrap());
        memory
            .remember(remember("k", "org", "c", &["shared"]))
            .unwrap();
        memory
            .remember(remember("k", "case-7", "d", &["shared", "only-case"]))
            .unwrap();
        memory.prune("case-7").unwrap();
    }
    let db = relate::Database::open(path.to_str().unwrap()).unwrap();
    let mut conn = db.connect().unwrap();
    let rows = conn
        .query("MATCH (t:Subject) RETURN t.name AS name ORDER BY name")
        .unwrap();
    let names: Vec<String> = rows
        .rows
        .iter()
        .map(|row| match &row[0] {
            relate::Value::String(s) => s.to_string(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(names, vec!["shared"]);
    let scopes = conn.query("MATCH (s:Scope) RETURN s.name").unwrap();
    assert_eq!(scopes.rows.len(), 1);
}
