//! The commands of examples/incident, with the full JSON of every step.

mod common;

use common::*;

const RUNBOOK: &str =
    "When checkout is down after a deploy, drain the checkout pool, then roll back the deploy.";
const RUNBOOK_SRC: &str =
    "INC-311: draining and rolling back restored checkout in 12 minutes. PM-14.";

fn seed(store: &Store) {
    store.set_agent(Some("oncall"));
    store.run(&[
        "remember", "severity-guide", "--claim",
        "Sev1 means checkout or login is down for every customer. Sev2 means a feature is degraded. Anything else is sev3.",
        "--about", "severity", "--about", "triage", "--scope", "org", "--because", "Incident policy IP-2, 2026-03-01.",
    ]);
}

fn seed_runbook(store: &Store) {
    store.run(&[
        "remember",
        "checkout-runbook",
        "--claim",
        RUNBOOK,
        "--about",
        "checkout",
        "--about",
        "runbook",
        "--scope",
        "org",
        "--because",
        RUNBOOK_SRC,
    ]);
}

#[test]
fn run_sev1_resolved() {
    let store = Store::new();
    seed(&store);
    let mut t = Transcript::new(&store);
    let guide = t.run(&[
        "recall",
        "--about",
        "severity",
        "--scope",
        "case-inc-311",
        "--scope",
        "org",
    ]);
    assert_eq!(guide["memories"][0]["key"], "severity-guide");
    let none = t.run(&[
        "recall",
        "--about",
        "checkout",
        "--about",
        "runbook",
        "--scope",
        "case-inc-311",
        "--scope",
        "org",
    ]);
    assert_eq!(none["action"], "none");
    t.run(&[
        "remember",
        "checkout-runbook",
        "--claim",
        RUNBOOK,
        "--about",
        "checkout",
        "--about",
        "runbook",
        "--scope",
        "org",
        "--because",
        RUNBOOK_SRC,
    ]);
    store.advance("10d");
    t.run(&[
        "recall",
        "--about",
        "checkout",
        "--about",
        "runbook",
        "--scope",
        "case-inc-320",
        "--scope",
        "org",
    ]);
    assert_eq!(t.last_action(), "use");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_sev2_unresolved() {
    let store = Store::new();
    seed(&store);
    seed_runbook(&store);
    let mut t = Transcript::new(&store);
    t.run(&[
        "recall",
        "--about",
        "checkout",
        "--about",
        "runbook",
        "--scope",
        "case-inc-312",
        "--scope",
        "org",
    ]);
    store.advance("3h");
    let fixed = t.run(&[
        "correct", "checkout-runbook", "--scope", "org", "--claim",
        "When checkout is down after a deploy, drain the checkout pool, then roll back the deploy. If search is still slow 15 minutes later, page the search vendor.",
        "--about", "checkout", "--about", "runbook", "--about", "search",
        "--because", "INC-312: the rollback did not fix search. The vendor fixed it 20 minutes after the page.",
    ]);
    assert_eq!(fixed["generation"], 2);
    store.advance("2d");
    let out = t.run(&["recall", "--about", "search", "--about", "runbook"]);
    assert_eq!(
        out["memories"][0]["about"],
        serde_json::json!(["checkout", "runbook", "search"])
    );
    assert_eq!(t.last_action(), "use");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_sev3() {
    let store = Store::new();
    seed(&store);
    store.run(&[
        "remember",
        "chart-refresh",
        "--claim",
        "A stale chart is fixed by refreshing the chart cache from the dashboards admin page.",
        "--about",
        "dashboards",
        "--about",
        "runbook",
        "--scope",
        "org",
        "--because",
        "INC-288: refreshing the cache fixed the revenue chart.",
    ]);
    store.advance("45d");
    let mut t = Transcript::new(&store);
    t.run(&["recall", "--about", "severity"]);
    let out = t.run(&[
        "recall",
        "--about",
        "dashboards",
        "--about",
        "runbook",
        "--fresh",
        "30d",
    ]);
    assert_eq!(out["stale"], serde_json::json!(["chart-refresh"]));
    assert_eq!(t.last_action(), "stale");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_retry() {
    let store = Store::new();
    seed(&store);
    let case = "case-inc-313";
    let mut t = Transcript::new(&store);
    t.run(&[
        "remember",
        "rollback-blocked",
        "--claim",
        "The rollback is refused while the deploy lock is held.",
        "--about",
        "checkout",
        "--about",
        "rollback",
        "--scope",
        case,
        "--because",
        "mitigate attempt 1 failed with \"rollback refused\".",
    ]);
    store.advance("20m");
    t.run(&["recall", "--about", "rollback", "--scope", case]);
    let gone = t.run(&[
        "forget",
        "rollback-blocked",
        "--scope",
        case,
        "--because",
        "The lock cleared and the retried rollback stuck.",
    ]);
    assert_eq!(gone["forgotten"], true);
    let again = t.run(&[
        "forget",
        "rollback-blocked",
        "--scope",
        case,
        "--because",
        "The lock cleared and the retried rollback stuck.",
    ]);
    assert_eq!(again["forgotten"], false);
    t.run(&["recall", "--about", "rollback", "--scope", case]);
    assert_eq!(t.last_action(), "none");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_replan() {
    let store = Store::new();
    seed(&store);
    seed_runbook(&store);
    for (key, claim, subject) in [
        (
            "checkout-owners",
            "The payments team owns checkout. Their escalation alias is payments-oncall.",
            "owners",
        ),
        (
            "checkout-dashboards",
            "Checkout latency and error rate are on the checkout-golden dashboard.",
            "dashboards",
        ),
        (
            "checkout-vendor",
            "The checkout payment gateway vendor is paged through the vendor portal, priority P1.",
            "vendor",
        ),
        (
            "checkout-flags",
            "The checkout-v2 flag can be turned off without a deploy.",
            "flags",
        ),
        (
            "checkout-db",
            "Checkout writes go to the orders-primary database.",
            "database",
        ),
    ] {
        store.run(&[
            "remember",
            key,
            "--claim",
            claim,
            "--about",
            "checkout",
            "--about",
            subject,
            "--scope",
            "org",
            "--because",
            "Checkout service catalog, 2026-11-01.",
        ]);
    }
    let mut t = Transcript::new(&store);
    t.run(&["recall", "--key", "checkout-runbook", "--scope", "org"]);
    let wide = t.run(&["recall", "--about", "checkout"]);
    assert_eq!(wide["matched"], 6);
    assert_eq!(t.last_action(), "narrow");
    insta::assert_json_snapshot!(t.steps());

    let vendor = store.run(&["recall", "--about", "checkout", "--about", "vendor"]);
    assert_eq!(vendor["memories"][0]["key"], "checkout-vendor");
}
