//! Properties that hold after any sequence of writes.

mod support;

use std::collections::BTreeMap;

use proptest::prelude::*;
use relate_semantic_core::{Recall, MAX_LIMIT};
use serde_json::Value;
use support::op::*;
use support::open;

fn history(memory: &relate_semantic_core::Memory, key: &str) -> Vec<Value> {
    match memory.log(key, None) {
        Ok(log) => log["generations"].as_array().unwrap().clone(),
        Err(err) => {
            assert_eq!(err.code, "no_memory");
            Vec::new()
        }
    }
}

fn check(memory: &relate_semantic_core::Memory) {
    let mut generations = 0;
    let mut live_total = 0;
    for key in KEYS {
        let rows = history(memory, key);
        generations += rows.len();
        let mut by_scope: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
        for row in &rows {
            by_scope
                .entry(row["scope"].as_str().unwrap().into())
                .or_default()
                .push(row);
        }
        for (scope, rows) in &by_scope {
            let mut numbers: Vec<u64> = rows
                .iter()
                .map(|r| r["generation"].as_u64().unwrap())
                .collect();
            numbers.sort();
            let expected: Vec<u64> = (1..=numbers.len() as u64).collect();
            assert_eq!(numbers, expected, "{key} in {scope} has gaps");

            let live: Vec<&&Value> = rows.iter().filter(|r| r["live"] == true).collect();
            assert!(live.len() <= 1, "{key} is live twice in {scope}");
            if let Some(live) = live.first() {
                live_total += 1;
                assert_eq!(
                    live["generation"].as_u64().unwrap(),
                    numbers.len() as u64,
                    "only the newest is live"
                );
                assert_eq!(live["forgotten"], false);
                assert!(live["claim"].is_string());
                assert!(!live["about"].as_array().unwrap().is_empty());
            }
            for row in rows {
                if row["forgotten"] == true {
                    assert!(row["claim"].is_null());
                    assert!(row["about"].as_array().unwrap().is_empty());
                    assert_eq!(row["live"], false);
                }
            }
        }

        match memory.show(key) {
            Ok(shown) => {
                let scopes: Vec<&str> = shown["memories"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|m| m["scope"].as_str().unwrap())
                    .collect();
                let live_scopes: Vec<&str> = by_scope
                    .iter()
                    .filter(|(_, rows)| rows.iter().any(|r| r["live"] == true))
                    .map(|(s, _)| s.as_str())
                    .collect();
                assert_eq!(scopes, live_scopes);
            }
            Err(err) => {
                assert_eq!(err.code, "no_memory");
                assert!(rows.iter().all(|r| r["live"] == false));
            }
        }
    }

    let listing = memory.scopes().unwrap();
    let listed: Vec<&Value> = listing["scopes"].as_array().unwrap().iter().collect();
    let sum = |field: &str| {
        listed
            .iter()
            .map(|s| s[field].as_u64().unwrap() as usize)
            .sum::<usize>()
    };
    assert_eq!(sum("generations"), generations);
    assert_eq!(sum("live"), live_total);

    for subject in SUBJECTS {
        for scope in SCOPES {
            let out = memory
                .recall(Recall {
                    about: vec![subject.to_string()],
                    keys: vec![],
                    scopes: vec![scope.to_string()],
                    fresh: None,
                    limit: MAX_LIMIT,
                })
                .unwrap();
            match out["action"].as_str().unwrap() {
                "use" => {
                    for m in out["memories"].as_array().unwrap() {
                        assert_eq!(m["scope"], *scope);
                        assert!(m["about"].as_array().unwrap().iter().any(|s| s == subject));
                    }
                }
                "none" => {}
                other => panic!("one scope and a high limit cannot give {other}"),
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

    #[test]
    fn invariants_hold_after_every_operation(ops in ops(24)) {
        let memory = open();
        for op in &ops {
            match run(&memory, op) {
                Ok(_) => {}
                Err(err) => prop_assert!(
                    matches!(err.code, "key_taken" | "no_memory"),
                    "unexpected refusal {err} after {op:?}"
                ),
            }
            check(&memory);
        }
    }

    #[test]
    fn every_write_repeats_without_writing(ops in ops(12)) {
        let memory = open();
        for op in &ops {
            let first = run(&memory, op);
            let before = memory.scopes().unwrap();
            let again = run(&memory, op);
            let after = memory.scopes().unwrap();
            if let (Ok(first), Ok(again)) = (&first, &again) {
                if first.get("forgotten") != Some(&Value::Bool(true)) && first.get("pruned").is_none() {
                    prop_assert_eq!(first, again);
                }
            }
            prop_assert_eq!(before, after, "repeating {:?} wrote again", op);
        }
    }
}
