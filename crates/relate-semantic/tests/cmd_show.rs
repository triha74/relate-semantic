mod common;

use common::*;

#[test]
fn show_prints_the_live_claim_in_every_scope() {
    let store = Store::new();
    store.price();
    store.note("widget-price", "case-inv-1042", &["northwind"]);
    let out = store.run(&["show", "widget-price"]);
    insta::assert_json_snapshot!(redact(&out));
}

#[test]
fn show_on_a_missing_key_suggests_remember() {
    let store = Store::new();
    let err = store.fail(&["show", "widget-price"], 4);
    assert_eq!(err["error"]["code"], "no_memory");
    assert_eq!(
        err["error"]["fix"],
        "relate-semantic remember widget-price --claim TEXT --about SUBJ --scope SCOPE --because TEXT"
    );
    assert!(!store.exists());
}

#[test]
fn show_on_a_forgotten_key_suggests_log() {
    let store = Store::new();
    store.price();
    store.run(&[
        "forget",
        "widget-price",
        "--scope",
        "org",
        "--because",
        "withdrawn",
    ]);
    let err = store.fail(&["show", "widget-price"], 4);
    assert_eq!(
        err["error"]["message"],
        "widget-price was forgotten in every scope"
    );
    assert_eq!(err["error"]["fix"], "relate-semantic log widget-price");
}

#[test]
fn show_validates_the_key_and_needs_one() {
    let store = Store::new();
    let err = store.fail(&["show", "Widget"], 4);
    assert_eq!(err["error"]["fix"], "relate-semantic show widget");
    let err = store.fail(&["show"], 2);
    assert_eq!(err["error"]["fix"], "relate-semantic show KEY");
}
