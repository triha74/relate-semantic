use serde_json::{json, Value};

/// One generation of one memory. Generations are never changed after they are
/// written, except that the live one stops being live when the next is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generation {
    pub key: String,
    pub scope: String,
    pub generation: u64,
    /// `None` on a tombstone written by `forget`.
    pub claim: Option<String>,
    pub because: String,
    /// Sorted, distinct subjects. Empty on a tombstone.
    pub about: Vec<String>,
    pub agent: String,
    pub at: String,
    pub forgotten: bool,
    pub live: bool,
}

impl Generation {
    /// The store-wide identity, held unique by a Relate constraint. Two writes
    /// that would both create the same generation of one key in one scope
    /// collide at commit.
    pub fn slot(&self) -> String {
        slot(&self.scope, &self.key, self.generation)
    }

    /// The claim as `recall` and `show` print it.
    pub fn memory_json(&self) -> Value {
        json!({
            "key": self.key,
            "scope": self.scope,
            "claim": self.claim,
            "because": self.because,
            "about": self.about,
            "agent": self.agent,
            "generation": self.generation,
            "at": self.at,
        })
    }

    /// What `remember` and `correct` print.
    pub fn written_json(&self) -> Value {
        json!({
            "key": self.key,
            "scope": self.scope,
            "generation": self.generation,
            "at": self.at,
        })
    }

    /// What `forget` prints after it writes a tombstone.
    pub fn forgotten_json(&self) -> Value {
        json!({
            "key": self.key,
            "scope": self.scope,
            "generation": self.generation,
            "at": self.at,
            "forgotten": true,
        })
    }

    /// One row of `log`.
    pub fn log_json(&self) -> Value {
        json!({
            "key": self.key,
            "scope": self.scope,
            "generation": self.generation,
            "claim": self.claim,
            "because": self.because,
            "about": self.about,
            "agent": self.agent,
            "at": self.at,
            "forgotten": self.forgotten,
            "live": self.live,
        })
    }
}

pub fn slot(scope: &str, key: &str, generation: u64) -> String {
    format!("{scope}/{key}/{generation}")
}

/// Sorted and distinct, so two calls that name the same subjects in a different
/// order are the same write.
pub fn normalize(subjects: &[String]) -> Vec<String> {
    let mut out = subjects.to_vec();
    out.sort();
    out.dedup();
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Write {
    /// A new generation. `supersedes` is the slot it replaces.
    Create {
        generation: Generation,
        supersedes: Option<String>,
    },
    /// The live generation in `slot` stops being live.
    Retire { slot: String },
    /// Delete every generation in one scope, the scope, and subjects left with
    /// no memory.
    Prune { scope: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Generation {
        Generation {
            key: "widget-price".into(),
            scope: "org".into(),
            generation: 2,
            claim: Some("fact".into()),
            because: "source".into(),
            about: vec!["a".into()],
            agent: "invoices".into(),
            at: "2026-11-10T18:00:00Z".into(),
            forgotten: false,
            live: true,
        }
    }

    #[test]
    fn json_shapes() {
        let g = sample();
        assert_eq!(g.slot(), "org/widget-price/2");
        assert_eq!(g.memory_json()["claim"], "fact");
        assert_eq!(g.written_json()["generation"], 2);
        assert!(g.written_json().get("claim").is_none());
        assert_eq!(g.forgotten_json()["forgotten"], true);
        assert_eq!(g.log_json()["live"], true);
    }

    #[test]
    fn subjects_are_sorted_and_distinct() {
        let raw: Vec<String> = ["b", "a", "b"].iter().map(|s| s.to_string()).collect();
        assert_eq!(normalize(&raw), vec!["a".to_string(), "b".to_string()]);
    }
}
