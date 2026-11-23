//! Every verb as a rule over the loaded [`State`]. Writes are pushed onto the
//! state and applied by the caller in the same transaction.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::clock::stamp;
use crate::error::MemoryError;
use crate::fix::{self, KEY, SCOPE, SUBJ};
use crate::limits::{LOG_CAP, MAX_LIMIT};
use crate::model::{normalize, Generation, Write};
use crate::recall::{decide, Recall};
use crate::store::State;
use crate::validate::Check;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remember {
    pub key: String,
    pub scope: String,
    pub claim: String,
    pub because: String,
    pub about: Vec<String>,
    pub agent: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correct {
    pub key: String,
    pub scope: String,
    pub claim: String,
    pub because: String,
    /// `None` keeps the live generation's subjects.
    pub about: Option<Vec<String>>,
    pub agent: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Forget {
    pub key: String,
    pub scope: String,
    pub because: String,
    pub agent: String,
}

/// The corrected agent, only when the given one was not a slug.
fn agent_fix(check: &mut Check, agent: &str) -> Option<String> {
    let fixed = check.slug("agent", Some("agent"), agent, "NAME");
    (fixed != agent).then_some(fixed)
}

pub fn remember(
    state: &mut State,
    spec: Remember,
    now: DateTime<Utc>,
) -> Result<Value, MemoryError> {
    let mut check = Check::new();
    let key = check.slug("key", None, &spec.key, KEY);
    let about = check.slugs("about", "about", &spec.about, SUBJ);
    check.required("about", "about", &spec.about, "remember");
    let scope = check.slug("scope", Some("scope"), &spec.scope, SCOPE);
    check.text("claim", &spec.claim);
    check.text("because", &spec.because);
    let by = agent_fix(&mut check, &spec.agent);
    check.finish(fix::remember(&key, &about, &scope, by.as_deref()))?;

    let about = normalize(&spec.about);
    if let Some(live) = state.live_in(&spec.key, &spec.scope) {
        if live.claim.as_deref() == Some(spec.claim.as_str())
            && live.because == spec.because
            && live.about == about
        {
            return Ok(live.written_json());
        }
        let changed = (live.about != about).then_some(about.as_slice());
        return Err(MemoryError::refusal(
            "key_taken",
            format!(
                "{} is already live in {}: \"{}\"",
                spec.key,
                spec.scope,
                clip(live.claim.as_deref().unwrap_or(""))
            ),
            fix::correct(&spec.key, &spec.scope, changed, None),
        ));
    }
    let previous = state.last_in(&spec.key, &spec.scope).cloned();
    let generation = Generation {
        key: spec.key,
        scope: spec.scope,
        generation: previous.as_ref().map_or(1, |p| p.generation + 1),
        claim: Some(spec.claim),
        because: spec.because,
        about,
        agent: spec.agent,
        at: stamp(now),
        forgotten: false,
        live: true,
    };
    let out = generation.written_json();
    state.push(Write::Create {
        generation,
        supersedes: previous.map(|p| p.slot()),
    });
    Ok(out)
}

pub fn correct(
    state: &mut State,
    mut spec: Correct,
    now: DateTime<Utc>,
) -> Result<Value, MemoryError> {
    spec.about = spec.about.filter(|about| !about.is_empty());
    let mut check = Check::new();
    let key = check.slug("key", None, &spec.key, KEY);
    let scope = check.slug("scope", Some("scope"), &spec.scope, SCOPE);
    check.text("claim", &spec.claim);
    check.text("because", &spec.because);
    let about = spec
        .about
        .as_ref()
        .map(|about| check.slugs("about", "about", about, SUBJ));
    let by = agent_fix(&mut check, &spec.agent);
    check.finish(fix::correct(&key, &scope, about.as_deref(), by.as_deref()))?;

    let Some(live) = state.live_in(&spec.key, &spec.scope).cloned() else {
        let subjects = spec
            .about
            .clone()
            .or_else(|| {
                state
                    .last_in(&spec.key, &spec.scope)
                    .map(|g| g.about.clone())
            })
            .unwrap_or_default();
        let forgotten = state.last_in(&spec.key, &spec.scope).is_some();
        return Err(MemoryError::refusal(
            "no_memory",
            if forgotten {
                format!("{} was forgotten in {}", spec.key, spec.scope)
            } else {
                format!("{} is not live in {}", spec.key, spec.scope)
            },
            fix::remember(&spec.key, &normalize(&subjects), &spec.scope, None),
        ));
    };
    let about = spec
        .about
        .as_deref()
        .map(normalize)
        .unwrap_or_else(|| live.about.clone());
    if live.claim.as_deref() == Some(spec.claim.as_str())
        && live.because == spec.because
        && live.about == about
    {
        return Ok(live.written_json());
    }
    let generation = Generation {
        key: spec.key,
        scope: spec.scope,
        generation: live.generation + 1,
        claim: Some(spec.claim),
        because: spec.because,
        about,
        agent: spec.agent,
        at: stamp(now),
        forgotten: false,
        live: true,
    };
    let out = generation.written_json();
    state.push(Write::Retire { slot: live.slot() });
    state.push(Write::Create {
        generation,
        supersedes: Some(live.slot()),
    });
    Ok(out)
}

pub fn forget(state: &mut State, spec: Forget, now: DateTime<Utc>) -> Result<Value, MemoryError> {
    let mut check = Check::new();
    let key = check.slug("key", None, &spec.key, KEY);
    let scope = check.slug("scope", Some("scope"), &spec.scope, SCOPE);
    check.text("because", &spec.because);
    let by = agent_fix(&mut check, &spec.agent);
    check.finish(fix::forget(&key, &scope, by.as_deref()))?;

    let Some(live) = state.live_in(&spec.key, &spec.scope).cloned() else {
        return Ok(json!({"key": spec.key, "scope": spec.scope, "forgotten": false}));
    };
    let tombstone = Generation {
        key: spec.key,
        scope: spec.scope,
        generation: live.generation + 1,
        claim: None,
        because: spec.because,
        about: Vec::new(),
        agent: spec.agent,
        at: stamp(now),
        forgotten: true,
        live: false,
    };
    let out = tombstone.forgotten_json();
    state.push(Write::Retire { slot: live.slot() });
    state.push(Write::Create {
        generation: tombstone,
        supersedes: Some(live.slot()),
    });
    Ok(out)
}

pub fn recall(state: &State, query: Recall, now: DateTime<Utc>) -> Result<Value, MemoryError> {
    if query.about.is_empty() && query.keys.is_empty() {
        return Err(MemoryError::usage(
            "recall needs at least one --about or --key",
            fix::recall(&[], &[], &query.scopes),
        ));
    }
    if query.limit == 0 || query.limit > MAX_LIMIT {
        return Err(MemoryError::usage(
            format!("--limit {} is outside 1 to {MAX_LIMIT}", query.limit),
            fix::recall(&query.about, &query.keys, &query.scopes),
        ));
    }
    let mut check = Check::new();
    let about = check.slugs("about", "about", &query.about, SUBJ);
    let keys = check.slugs("key", "key", &query.keys, KEY);
    let scopes = check.slugs("scope", "scope", &query.scopes, SCOPE);
    check.finish(fix::recall(&about, &keys, &scopes))?;

    let live: Vec<&Generation> = state.live().collect();
    Ok(decide(&live, &query, now))
}

pub fn show(state: &State, key: &str) -> Result<Value, MemoryError> {
    let mut check = Check::new();
    let fixed = check.slug("key", None, key, KEY);
    check.finish(fix::show(&fixed))?;
    let mut live: Vec<&Generation> = state.live().filter(|g| g.key == key).collect();
    if live.is_empty() {
        let history = state.generations.iter().any(|g| g.key == key);
        return Err(MemoryError::refusal(
            "no_memory",
            if history {
                format!("{key} was forgotten in every scope")
            } else {
                format!("{key} is not live in any scope")
            },
            if history {
                fix::log(key, None)
            } else {
                fix::remember(key, &[], SCOPE, None)
            },
        ));
    }
    live.sort_by(|a, b| a.scope.cmp(&b.scope));
    Ok(json!({
        "key": key,
        "memories": live.iter().map(|g| g.memory_json()).collect::<Vec<_>>(),
    }))
}

pub fn log(state: &State, key: &str, scope: Option<&str>) -> Result<Value, MemoryError> {
    let mut check = Check::new();
    let fixed = check.slug("key", None, key, KEY);
    let fixed_scope = scope.map(|s| check.slug("scope", Some("scope"), s, SCOPE));
    check.finish(fix::log(&fixed, fixed_scope.as_deref()))?;
    let mut rows: Vec<&Generation> = state
        .generations
        .iter()
        .filter(|g| g.key == key && scope.is_none_or(|s| g.scope == s))
        .collect();
    if rows.is_empty() {
        return Err(MemoryError::refusal(
            "no_memory",
            match scope {
                Some(scope) => format!("{key} has no generations in {scope}"),
                None => format!("{key} has no generations"),
            },
            fix::remember(key, &[], scope.unwrap_or(SCOPE), None),
        ));
    }
    rows.sort_by(|a, b| {
        (b.at.as_str(), b.generation, b.scope.as_str()).cmp(&(
            a.at.as_str(),
            a.generation,
            a.scope.as_str(),
        ))
    });
    rows.truncate(LOG_CAP);
    Ok(json!({
        "key": key,
        "generations": rows.iter().map(|g| g.log_json()).collect::<Vec<_>>(),
    }))
}

pub fn prune(state: &mut State, scope: &str) -> Result<Value, MemoryError> {
    let mut check = Check::new();
    let fixed = check.slug("scope", Some("scope"), scope, SCOPE);
    check.finish(fix::prune(&fixed))?;
    let generations: Vec<&Generation> = state
        .generations
        .iter()
        .filter(|g| g.scope == scope)
        .collect();
    let keys: BTreeSet<&str> = generations.iter().map(|g| g.key.as_str()).collect();
    let out = json!({
        "pruned": scope,
        "keys": keys.len(),
        "generations": generations.len(),
    });
    if !generations.is_empty() {
        state.push(Write::Prune {
            scope: scope.to_string(),
        });
    }
    Ok(out)
}

pub fn scopes(state: &State) -> Value {
    let mut table: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for g in &state.generations {
        let entry = table.entry(g.scope.as_str()).or_default();
        entry.1 += 1;
        if g.live {
            entry.0 += 1;
        }
    }
    json!({
        "scopes": table
            .into_iter()
            .map(|(name, (live, generations))| json!({"name": name, "live": live, "generations": generations}))
            .collect::<Vec<_>>(),
    })
}

fn clip(text: &str) -> String {
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(80).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_keeps_short_text_and_marks_long_text() {
        assert_eq!(clip("short"), "short");
        let long = "x".repeat(100);
        assert!(clip(&long).ends_with('…'));
        assert_eq!(clip(&long).chars().count(), 81);
    }
}
