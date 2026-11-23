//! Random operation sequences over small alphabets, so keys, scopes, and
//! claims collide often.

use proptest::prelude::*;
use relate_semantic_core::{Correct, Forget, Memory, MemoryError, Remember};
use serde_json::Value;

pub const KEYS: &[&str] = &["price", "terms"];
pub const SCOPES: &[&str] = &["org", "case-7"];
pub const SUBJECTS: &[&str] = &["nw", "acme", "discounts"];
pub const CLAIMS: &[&str] = &["c1", "c2"];
pub const BECAUSE: &str = "test";

#[derive(Debug, Clone)]
pub enum Op {
    Remember {
        key: usize,
        scope: usize,
        claim: usize,
        about: Vec<usize>,
    },
    Correct {
        key: usize,
        scope: usize,
        claim: usize,
        about: Option<Vec<usize>>,
    },
    Forget {
        key: usize,
        scope: usize,
    },
    Prune {
        scope: usize,
    },
}

fn about() -> impl Strategy<Value = Vec<usize>> {
    prop::collection::btree_set(0..SUBJECTS.len(), 1..=SUBJECTS.len())
        .prop_map(|s| s.into_iter().collect())
}

pub fn op() -> impl Strategy<Value = Op> {
    let k = 0..KEYS.len();
    let s = 0..SCOPES.len();
    let c = 0..CLAIMS.len();
    prop_oneof![
        4 => (k.clone(), s.clone(), c.clone(), about())
            .prop_map(|(key, scope, claim, about)| Op::Remember { key, scope, claim, about }),
        4 => (k.clone(), s.clone(), c, prop::option::of(about()))
            .prop_map(|(key, scope, claim, about)| Op::Correct { key, scope, claim, about }),
        2 => (k, s.clone()).prop_map(|(key, scope)| Op::Forget { key, scope }),
        1 => s.prop_map(|scope| Op::Prune { scope }),
    ]
}

pub fn ops(max: usize) -> impl Strategy<Value = Vec<Op>> {
    prop::collection::vec(op(), 1..max)
}

pub fn names(indices: &[usize]) -> Vec<String> {
    indices.iter().map(|&i| SUBJECTS[i].to_string()).collect()
}

pub fn run(memory: &Memory, op: &Op) -> Result<Value, MemoryError> {
    match op {
        Op::Remember {
            key,
            scope,
            claim,
            about,
        } => memory.remember(Remember {
            key: KEYS[*key].into(),
            scope: SCOPES[*scope].into(),
            claim: CLAIMS[*claim].into(),
            because: BECAUSE.into(),
            about: names(about),
            agent: "prop".into(),
        }),
        Op::Correct {
            key,
            scope,
            claim,
            about,
        } => memory.correct(Correct {
            key: KEYS[*key].into(),
            scope: SCOPES[*scope].into(),
            claim: CLAIMS[*claim].into(),
            because: BECAUSE.into(),
            about: about.as_deref().map(names),
            agent: "prop".into(),
        }),
        Op::Forget { key, scope } => memory.forget(Forget {
            key: KEYS[*key].into(),
            scope: SCOPES[*scope].into(),
            because: BECAUSE.into(),
            agent: "prop".into(),
        }),
        Op::Prune { scope } => memory.prune(SCOPES[*scope]),
    }
}
