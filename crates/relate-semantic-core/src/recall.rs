//! The recall decision. A pure function of the live generations, the query,
//! and the time. It never sees a connection.

use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::clock::parse_stamp;
use crate::fix;
use crate::model::Generation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recall {
    /// Every subject must be on the memory.
    pub about: Vec<String>,
    /// When not empty, the key must be one of these.
    pub keys: Vec<String>,
    /// Ordered. Empty means every scope, with no order.
    pub scopes: Vec<String>,
    pub fresh: Option<Fresh>,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fresh {
    pub duration: Duration,
    /// The duration as the agent wrote it, echoed in a `stale` answer.
    pub label: String,
}

pub fn decide(live: &[&Generation], query: &Recall, now: DateTime<Utc>) -> Value {
    let mut by_key: BTreeMap<&str, Vec<&Generation>> = BTreeMap::new();
    for g in live.iter().copied().filter(|g| matches(g, query)) {
        by_key.entry(g.key.as_str()).or_default().push(g);
    }

    if query.scopes.is_empty() {
        if let Some((key, gens)) = by_key.iter().find(|(_, gens)| gens.len() > 1) {
            let mut scopes: Vec<String> = gens.iter().map(|g| g.scope.clone()).collect();
            scopes.sort();
            return json!({
                "action": "conflict",
                "key": key,
                "scopes": scopes,
                "fix": fix::recall(&[], &[key.to_string()], &scopes),
            });
        }
    }

    let mut winners: Vec<&Generation> = by_key
        .values()
        .map(|gens| {
            *gens
                .iter()
                .min_by_key(|g| scope_rank(query, &g.scope))
                .expect("a key has at least one generation")
        })
        .collect();

    if winners.len() > query.limit {
        let keys: Vec<Value> = winners
            .iter()
            .map(|g| json!({"key": g.key, "scope": g.scope, "about": g.about}))
            .collect();
        let first = winners[0].key.clone();
        return json!({
            "action": "narrow",
            "matched": winners.len(),
            "limit": query.limit,
            "keys": keys,
            "fix": fix::recall(&query.about, &[first], &query.scopes),
        });
    }

    winners.sort_by_key(|g| {
        (
            scope_rank(query, &g.scope),
            Reverse(g.at.clone()),
            g.key.clone(),
        )
    });
    let memories: Vec<Value> = winners.iter().map(|g| g.memory_json()).collect();

    if let Some(fresh) = &query.fresh {
        let mut stale: Vec<String> = winners
            .iter()
            .filter(|g| is_stale(g, fresh.duration, now))
            .map(|g| g.key.clone())
            .collect();
        if !stale.is_empty() {
            stale.sort();
            return json!({
                "action": "stale",
                "stale": stale,
                "fresh": fresh.label,
                "memories": memories,
            });
        }
    }

    if !memories.is_empty() {
        return json!({"action": "use", "memories": memories});
    }

    json!({
        "action": "none",
        "about": query.about,
        "keys": query.keys,
        "scopes": query.scopes,
    })
}

fn matches(g: &Generation, query: &Recall) -> bool {
    (query.scopes.is_empty() || query.scopes.contains(&g.scope))
        && (query.keys.is_empty() || query.keys.contains(&g.key))
        && query.about.iter().all(|s| g.about.contains(s))
}

fn scope_rank(query: &Recall, scope: &str) -> usize {
    query.scopes.iter().position(|s| s == scope).unwrap_or(0)
}

fn is_stale(g: &Generation, fresh: Duration, now: DateTime<Utc>) -> bool {
    let Some(at) = parse_stamp(&g.at) else {
        return true;
    };
    let age = now.signed_duration_since(at);
    age.to_std().map(|age| age > fresh).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(key: &str, scope: &str, about: &[&str], at: &str) -> Generation {
        Generation {
            key: key.into(),
            scope: scope.into(),
            generation: 1,
            claim: Some(format!("{key} in {scope}")),
            because: "source".into(),
            about: about.iter().map(|s| s.to_string()).collect(),
            agent: "a".into(),
            at: at.into(),
            forgotten: false,
            live: true,
        }
    }

    fn q(about: &[&str], scopes: &[&str]) -> Recall {
        Recall {
            about: about.iter().map(|s| s.to_string()).collect(),
            keys: vec![],
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
            fresh: None,
            limit: 5,
        }
    }

    fn now() -> DateTime<Utc> {
        parse_stamp("2026-11-17T18:00:00Z").unwrap()
    }

    #[test]
    fn an_unreadable_stamp_is_stale() {
        let mut bad = g("k", "org", &["a"], "nope");
        bad.at = "nope".into();
        assert!(is_stale(&bad, Duration::from_secs(1), now()));
    }

    #[test]
    fn a_future_stamp_is_not_stale() {
        let later = g("k", "org", &["a"], "2027-01-01T00:00:00Z");
        assert!(!is_stale(&later, Duration::from_secs(1), now()));
    }

    #[test]
    fn subjects_are_and() {
        let both = g("both", "org", &["a", "b"], "2026-11-10T18:00:00Z");
        let one = g("one", "org", &["a"], "2026-11-10T18:00:00Z");
        let live = vec![&both, &one];
        let out = decide(&live, &q(&["a", "b"], &[]), now());
        assert_eq!(out["action"], "use");
        assert_eq!(out["memories"].as_array().unwrap().len(), 1);
        assert_eq!(out["memories"][0]["key"], "both");
    }
}
