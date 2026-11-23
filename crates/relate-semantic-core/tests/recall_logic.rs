//! The recall decision as a table. Each row builds the live generations that
//! trigger two actions at once and checks that the earlier action in the
//! order conflict, narrow, stale, use, none wins.

use std::time::Duration;

use relate_semantic_core::{decide, parse_stamp, Fresh, Generation, Recall};
use serde_json::json;

const NOW: &str = "2026-11-17T18:00:00Z";
const OLD: &str = "2026-10-01T18:00:00Z";
const NEW: &str = "2026-11-16T18:00:00Z";

fn g(key: &str, scope: &str, about: &[&str], at: &str) -> Generation {
    Generation {
        key: key.into(),
        scope: scope.into(),
        generation: 1,
        claim: Some(format!("{key} in {scope}")),
        because: "source".into(),
        about: about.iter().map(|s| s.to_string()).collect(),
        agent: "invoices".into(),
        at: at.into(),
        forgotten: false,
        live: true,
    }
}

fn q(about: &[&str], scopes: &[&str], fresh_days: Option<u64>, limit: usize) -> Recall {
    Recall {
        about: about.iter().map(|s| s.to_string()).collect(),
        keys: vec![],
        scopes: scopes.iter().map(|s| s.to_string()).collect(),
        fresh: fresh_days.map(|d| Fresh {
            duration: Duration::from_secs(d * 86_400),
            label: format!("{d}d"),
        }),
        limit,
    }
}

fn action(live: &[Generation], query: &Recall) -> String {
    let refs: Vec<&Generation> = live.iter().collect();
    decide(&refs, query, parse_stamp(NOW).unwrap())["action"]
        .as_str()
        .unwrap()
        .to_string()
}

/// Two scopes share a key, three keys match, and one is old.
fn crowded() -> Vec<Generation> {
    vec![
        g("price", "org", &["nw"], OLD),
        g("price", "case-7", &["nw"], NEW),
        g("terms", "org", &["nw"], NEW),
        g("contact", "org", &["nw"], NEW),
    ]
}

#[test]
fn the_decision_order_holds_pairwise() {
    let rows: &[(&str, Vec<Generation>, Recall, &str)] = &[
        (
            "conflict beats narrow",
            crowded(),
            q(&["nw"], &[], None, 1),
            "conflict",
        ),
        (
            "conflict beats stale",
            crowded(),
            q(&["nw"], &[], Some(7), 5),
            "conflict",
        ),
        (
            "conflict beats use",
            crowded(),
            q(&["nw"], &[], None, 5),
            "conflict",
        ),
        (
            "narrow beats stale",
            crowded(),
            q(&["nw"], &["org"], Some(7), 2),
            "narrow",
        ),
        (
            "narrow beats use",
            crowded(),
            q(&["nw"], &["org"], None, 2),
            "narrow",
        ),
        (
            "stale beats use",
            crowded(),
            q(&["nw"], &["org"], Some(7), 5),
            "stale",
        ),
        (
            "use when fresh",
            crowded(),
            q(&["nw"], &["case-7"], Some(7), 5),
            "use",
        ),
        (
            "use at the limit",
            crowded(),
            q(&["nw"], &["org"], None, 3),
            "use",
        ),
        (
            "none with no match",
            crowded(),
            q(&["other"], &[], Some(7), 1),
            "none",
        ),
        (
            "none for an unknown scope",
            crowded(),
            q(&["nw"], &["nobody"], None, 5),
            "none",
        ),
        (
            "none on an empty store",
            vec![],
            q(&["nw"], &[], None, 5),
            "none",
        ),
    ];
    for (name, live, query, expected) in rows {
        assert_eq!(action(live, query), *expected, "{name}");
    }
}

#[test]
fn a_scope_list_resolves_a_shared_key_to_the_earliest_scope() {
    let live = crowded();
    let refs: Vec<&Generation> = live.iter().collect();
    let now = parse_stamp(NOW).unwrap();
    let out = decide(&refs, &q(&["nw"], &["case-7", "org"], None, 5), now);
    assert_eq!(out["action"], "use");
    let picked: Vec<(&str, &str)> = out["memories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| (m["key"].as_str().unwrap(), m["scope"].as_str().unwrap()))
        .collect();
    assert_eq!(
        picked,
        vec![("price", "case-7"), ("contact", "org"), ("terms", "org")]
    );
    let out = decide(&refs, &q(&["nw"], &["org", "case-7"], None, 5), now);
    assert_eq!(out["memories"][0]["scope"], "org");
}

#[test]
fn use_orders_by_scope_then_newest_then_key() {
    let live = [
        g("b", "org", &["s"], OLD),
        g("a", "org", &["s"], OLD),
        g("c", "org", &["s"], NEW),
        g("d", "case-7", &["s"], OLD),
    ];
    let refs: Vec<&Generation> = live.iter().collect();
    let out = decide(
        &refs,
        &q(&["s"], &["case-7", "org"], None, 5),
        parse_stamp(NOW).unwrap(),
    );
    let keys: Vec<&str> = out["memories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["key"].as_str().unwrap())
        .collect();
    assert_eq!(keys, vec!["d", "c", "a", "b"]);
}

#[test]
fn each_action_has_its_documented_shape() {
    let now = parse_stamp(NOW).unwrap();
    let live = crowded();
    let refs: Vec<&Generation> = live.iter().collect();

    let conflict = decide(&refs, &q(&["nw"], &[], None, 5), now);
    assert_eq!(
        conflict,
        json!({
            "action": "conflict",
            "key": "price",
            "scopes": ["case-7", "org"],
            "fix": "relate-semantic recall --key price --scope case-7 --scope org",
        })
    );

    let narrow = decide(&refs, &q(&["nw"], &["org"], None, 2), now);
    assert_eq!(
        narrow,
        json!({
            "action": "narrow",
            "matched": 3,
            "limit": 2,
            "keys": [
                {"key": "contact", "scope": "org", "about": ["nw"]},
                {"key": "price", "scope": "org", "about": ["nw"]},
                {"key": "terms", "scope": "org", "about": ["nw"]},
            ],
            "fix": "relate-semantic recall --about nw --key contact --scope org",
        })
    );

    let stale = decide(&refs, &q(&["nw"], &["org"], Some(7), 5), now);
    assert_eq!(stale["stale"], json!(["price"]));
    assert_eq!(stale["fresh"], "7d");
    assert_eq!(stale["memories"].as_array().unwrap().len(), 3);

    let none = decide(&refs, &q(&["x"], &["org"], None, 5), now);
    assert_eq!(
        none,
        json!({"action": "none", "about": ["x"], "keys": [], "scopes": ["org"]})
    );
}

#[test]
fn keys_filter_and_combine_with_subjects() {
    let live = crowded();
    let refs: Vec<&Generation> = live.iter().collect();
    let now = parse_stamp(NOW).unwrap();
    let mut query = q(&[], &["org"], None, 5);
    query.keys = vec!["terms".into(), "contact".into()];
    let out = decide(&refs, &query, now);
    assert_eq!(out["memories"].as_array().unwrap().len(), 2);
    query.about = vec!["other".into()];
    assert_eq!(decide(&refs, &query, now)["action"], "none");
}

#[test]
fn a_memory_exactly_at_the_fresh_boundary_is_not_stale() {
    let live = vec![g("k", "org", &["s"], "2026-11-10T18:00:00Z")];
    assert_eq!(action(&live, &q(&["s"], &[], Some(7), 5)), "use");
    let live = vec![g("k", "org", &["s"], "2026-11-10T17:59:59Z")];
    assert_eq!(action(&live, &q(&["s"], &[], Some(7), 5)), "stale");
}
