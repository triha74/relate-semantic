//! The store against a reference model written with plain maps. Every
//! operation must give the same outcome in both, and the live claims must
//! agree after every step.

mod support;

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use relate_semantic_core::Memory;
use support::op::*;
use support::open;

#[derive(Debug, Clone, PartialEq)]
struct Entry {
    claim: Option<String>,
    about: Vec<String>,
}

/// (scope, key) to every generation, oldest first. The last entry is live
/// unless it is a tombstone.
#[derive(Default)]
struct Model {
    slots: BTreeMap<(String, String), Vec<Entry>>,
}

#[derive(Debug, PartialEq)]
enum Outcome {
    Wrote(u64),
    Same(u64),
    Refused(&'static str),
    NotForgotten,
    Pruned(usize, usize),
}

fn sorted(about: Vec<String>) -> Vec<String> {
    about
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

impl Model {
    fn live(&self, scope: &str, key: &str) -> Option<(u64, &Entry)> {
        let gens = self.slots.get(&(scope.into(), key.into()))?;
        let last = gens.last()?;
        last.claim.as_ref().map(|_| (gens.len() as u64, last))
    }

    fn push(&mut self, scope: &str, key: &str, entry: Entry) -> u64 {
        let gens = self.slots.entry((scope.into(), key.into())).or_default();
        gens.push(entry);
        gens.len() as u64
    }

    fn apply(&mut self, op: &Op) -> Outcome {
        match op {
            Op::Remember {
                key,
                scope,
                claim,
                about,
            } => {
                let (key, scope, claim) = (KEYS[*key], SCOPES[*scope], CLAIMS[*claim]);
                let about = sorted(names(about));
                if let Some((n, live)) = self.live(scope, key) {
                    return if live.claim.as_deref() == Some(claim) && live.about == about {
                        Outcome::Same(n)
                    } else {
                        Outcome::Refused("key_taken")
                    };
                }
                Outcome::Wrote(self.push(
                    scope,
                    key,
                    Entry {
                        claim: Some(claim.into()),
                        about,
                    },
                ))
            }
            Op::Correct {
                key,
                scope,
                claim,
                about,
            } => {
                let (key, scope, claim) = (KEYS[*key], SCOPES[*scope], CLAIMS[*claim]);
                let Some((n, live)) = self.live(scope, key) else {
                    return Outcome::Refused("no_memory");
                };
                let about = about
                    .as_deref()
                    .map(|a| sorted(names(a)))
                    .unwrap_or_else(|| live.about.clone());
                if live.claim.as_deref() == Some(claim) && live.about == about {
                    return Outcome::Same(n);
                }
                Outcome::Wrote(self.push(
                    scope,
                    key,
                    Entry {
                        claim: Some(claim.into()),
                        about,
                    },
                ))
            }
            Op::Forget { key, scope } => {
                let (key, scope) = (KEYS[*key], SCOPES[*scope]);
                if self.live(scope, key).is_none() {
                    return Outcome::NotForgotten;
                }
                Outcome::Wrote(self.push(
                    scope,
                    key,
                    Entry {
                        claim: None,
                        about: vec![],
                    },
                ))
            }
            Op::Prune { scope } => {
                let scope = SCOPES[*scope];
                let gone: Vec<(String, String)> = self
                    .slots
                    .keys()
                    .filter(|(s, _)| s == scope)
                    .cloned()
                    .collect();
                let generations = gone.iter().map(|slot| self.slots[slot].len()).sum();
                for slot in &gone {
                    self.slots.remove(slot);
                }
                Outcome::Pruned(gone.len(), generations)
            }
        }
    }
}

fn generations(memory: &Memory) -> u64 {
    memory.scopes().unwrap()["scopes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["generations"].as_u64().unwrap())
        .sum()
}

/// Run one operation and classify it. A generation number that did not add
/// a generation is a repeat.
fn observe(memory: &Memory, op: &Op) -> Outcome {
    let before = generations(memory);
    let value = match run(memory, op) {
        Err(err) => return Outcome::Refused(err.code),
        Ok(value) => value,
    };
    if value.get("pruned").is_some() {
        return Outcome::Pruned(
            value["keys"].as_u64().unwrap() as usize,
            value["generations"].as_u64().unwrap() as usize,
        );
    }
    if value.get("forgotten") == Some(&serde_json::Value::Bool(false)) {
        return Outcome::NotForgotten;
    }
    let n = value["generation"].as_u64().unwrap();
    if generations(memory) > before {
        Outcome::Wrote(n)
    } else {
        Outcome::Same(n)
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    #[test]
    fn the_store_matches_the_model(ops in ops(30)) {
        let memory = open();
        let mut model = Model::default();
        for op in &ops {
            let expected = model.apply(op);
            let actual = observe(&memory, op);
            prop_assert_eq!(&actual, &expected, "after {:?}", op);

            for key in KEYS {
                for scope in SCOPES {
                    let shown = memory.show(key).ok();
                    let claim = shown.as_ref().and_then(|s| {
                        s["memories"].as_array().unwrap().iter()
                            .find(|m| m["scope"] == *scope)
                            .map(|m| (m["generation"].as_u64().unwrap(), m["claim"].as_str().unwrap().to_string(),
                                      m["about"].as_array().unwrap().iter().map(|a| a.as_str().unwrap().to_string()).collect::<Vec<_>>()))
                    });
                    let modeled = model.live(scope, key).map(|(n, e)| (n, e.claim.clone().unwrap(), e.about.clone()));
                    prop_assert_eq!(claim, modeled, "{} in {}", key, scope);
                }
            }
        }
    }
}
