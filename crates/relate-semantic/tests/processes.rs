//! Separate processes on one store. The directory lock turns overlap into
//! `store_busy` after a short wait, or the second process runs after the
//! first. Either way the store ends with one live generation.

mod common;

use std::process::Stdio;

use common::*;

#[test]
fn concurrent_processes_never_leave_two_live_generations() {
    let store = Store::new();
    store.price();
    let children: Vec<_> = ["left", "right", "third"]
        .iter()
        .map(|claim| {
            let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_relate-semantic"));
            cmd.args([
                "correct",
                "widget-price",
                "--scope",
                "org",
                "--claim",
                claim,
                "--because",
                "race",
            ])
            .args(["-d", &store.path()])
            .env("RELATE_SEMANTIC_AGENT", "invoices")
            .env("RELATE_SEMANTIC_NOW", T0)
            .stdout(Stdio::piped());
            cmd.spawn().unwrap()
        })
        .collect();
    for child in children {
        let out = child.wait_with_output().unwrap();
        let value = parse(&out.stdout);
        match out.status.code() {
            Some(0) => assert!(value["generation"].as_u64().unwrap() >= 2),
            Some(5) => {
                assert!(matches!(
                    value["error"]["code"].as_str(),
                    Some("store_busy" | "conflict")
                ));
                assert_eq!(value["error"]["retryable"], true);
            }
            other => panic!("exit {other:?}: {value}"),
        }
    }
    let log = store.run(&["log", "widget-price"]);
    let live = log["generations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|g| g["live"] == true)
        .count();
    assert_eq!(live, 1);
}
