//! The commands of examples/onboarding, with the full JSON of every step.

mod common;

use common::*;

fn seed(store: &Store) {
    store.set_agent(Some("kyc"));
    store.run(&[
        "remember", "id-documents", "--claim",
        "Accept a passport or a national ID card as proof of identity. A driving licence is not enough.",
        "--about", "identity", "--about", "documents", "--scope", "org", "--because", "KYC policy K-4, 2026-02-01.",
    ]);
}

#[test]
fn run_clear() {
    let store = Store::new();
    seed(&store);
    let mut t = Transcript::new(&store);
    t.run(&[
        "recall",
        "--about",
        "identity",
        "--scope",
        "case-app-1",
        "--scope",
        "org",
    ]);
    t.run(&[
        "recall",
        "--about",
        "risk",
        "--about",
        "threshold",
        "--scope",
        "case-app-1",
        "--scope",
        "org",
    ]);
    t.run(&[
        "remember", "low-risk-threshold", "--claim",
        "Score an applicant low risk when identity is verified, sanctions are clear, and expected monthly volume is under $10,000.",
        "--about", "risk", "--about", "threshold", "--scope", "org",
        "--because", "Analyst rule applied to APP-1 on 2026-11-10.",
    ]);
    store.advance("1d");
    t.run(&[
        "recall",
        "--about",
        "risk",
        "--about",
        "threshold",
        "--scope",
        "case-app-6",
        "--scope",
        "org",
    ]);
    assert_eq!(t.last_action(), "use");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_mismatch_approved() {
    let store = Store::new();
    seed(&store);
    let mut t = Transcript::new(&store);
    t.run(&[
        "recall",
        "--about",
        "name-mismatch",
        "--scope",
        "case-app-2",
        "--scope",
        "org",
    ]);
    t.run(&[
        "remember", "transliterated-names", "--agent", "compliance", "--claim",
        "A name that differs from the document only by transliteration, such as Aleksandr and Alexander, is approved when the date of birth and document number match.",
        "--about", "name-mismatch", "--about", "transliteration", "--scope", "org",
        "--because", "Compliance approved APP-2 with the corrected name on 2026-11-10.",
    ]);
    store.advance("2d");
    let out = t.run(&[
        "recall",
        "--about",
        "name-mismatch",
        "--scope",
        "case-app-7",
        "--scope",
        "org",
    ]);
    assert_eq!(out["memories"][0]["agent"], "compliance");
    assert_eq!(t.last_action(), "use");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_hit_declined() {
    let store = Store::new();
    seed(&store);
    let mut t = Transcript::new(&store);
    t.run(&[
        "remember", "sanctioned-entity", "--agent", "compliance", "--claim",
        "Orion Trading LLC is on the EU sanctions list. Decline any application where it is a beneficial owner.",
        "--about", "orion-trading", "--about", "sanctions", "--scope", "org",
        "--because", "Screening hit on APP-3, confirmed by compliance on 2026-11-10.",
    ]);
    store.advance("5d");
    t.run(&[
        "recall",
        "--about",
        "orion-trading",
        "--scope",
        "case-app-8",
        "--scope",
        "org",
    ]);
    assert_eq!(t.last_action(), "use");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_blocked() {
    let store = Store::new();
    seed(&store);
    let case = "case-app-4";
    let because = "verify-identity blocked: the document image is unreadable.";
    let dump = format!("OCR: {}", "P<UTO#### ?? unreadable ## ".repeat(30));
    let claim = "The passport scan for APP-4 is unreadable. Ask the applicant for a new scan.";
    let mut t = Transcript::new(&store);
    let long = t.refused(
        "too_long",
        &[
            "remember",
            "app-4-document",
            "--claim",
            &dump,
            "--about",
            "passport-scan",
            "--scope",
            case,
            "--because",
            because,
        ],
    );
    assert_eq!(long["error"]["errors"][0]["field"], "claim");
    let slug = t.refused(
        "bad_slug",
        &[
            "remember",
            "app-4-document",
            "--claim",
            claim,
            "--about",
            "Passport Scan",
            "--scope",
            case,
            "--because",
            because,
        ],
    );
    assert_eq!(
        slug["error"]["fix"],
        "relate-semantic remember app-4-document --claim TEXT --about passport-scan --scope case-app-4 --because TEXT"
    );
    t.run(&[
        "remember",
        "app-4-document",
        "--claim",
        claim,
        "--about",
        "passport-scan",
        "--scope",
        case,
        "--because",
        because,
    ]);
    store.advance("1d");
    t.run(&["recall", "--about", "passport-scan", "--scope", case]);
    assert_eq!(t.last_action(), "use");
    insta::assert_json_snapshot!(t.steps());
}

#[test]
fn run_reopen_donewhen() {
    let store = Store::new();
    seed(&store);
    store.set_agent(Some("compliance"));
    let mut t = Transcript::new(&store);
    t.run(&[
        "remember",
        "address-proof",
        "--claim",
        "A utility bill under 3 months old is proof of address.",
        "--about",
        "address",
        "--about",
        "documents",
        "--scope",
        "org",
        "--because",
        "Compliance memo CM-3.",
    ]);
    t.run(&[
        "recall",
        "--about",
        "address",
        "--scope",
        "case-app-5",
        "--scope",
        "org",
    ]);
    store.advance("1d");
    t.run(&[
        "correct",
        "address-proof",
        "--scope",
        "org",
        "--claim",
        "A utility bill or a bank statement under 3 months old is proof of address.",
        "--about",
        "proof-of-address",
        "--about",
        "documents",
        "--because",
        "Compliance memo CM-4 replaced CM-3.",
    ]);
    t.run(&[
        "recall",
        "--about",
        "address",
        "--scope",
        "case-app-5",
        "--scope",
        "org",
    ]);
    assert_eq!(t.last_action(), "none");
    insta::assert_json_snapshot!(t.steps());

    let renamed = store.run(&["recall", "--about", "proof-of-address"]);
    assert_eq!(renamed["memories"][0]["generation"], 2);
}
