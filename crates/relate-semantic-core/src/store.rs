//! Load and save the memory graph. One transaction per command. Each
//! [`Write`] touches only the nodes that command changed.

use std::collections::BTreeMap;
use std::sync::Arc;

use relate::{Connection, Error as RelateError, Value};

use crate::error::MemoryError;
use crate::model::{Generation, Write};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub generations: Vec<Generation>,
    pub writes: Vec<Write>,
}

impl State {
    pub fn live(&self) -> impl Iterator<Item = &Generation> {
        self.generations.iter().filter(|g| g.live)
    }

    pub fn live_in(&self, key: &str, scope: &str) -> Option<&Generation> {
        self.live().find(|g| g.key == key && g.scope == scope)
    }

    /// The newest generation of a key in a scope, live or not.
    pub fn last_in(&self, key: &str, scope: &str) -> Option<&Generation> {
        self.generations
            .iter()
            .filter(|g| g.key == key && g.scope == scope)
            .max_by_key(|g| g.generation)
    }

    /// Record a write and apply it to the loaded state, so a later rule in the
    /// same command sees it.
    pub fn push(&mut self, write: Write) {
        match &write {
            Write::Create { generation, .. } => self.generations.push(generation.clone()),
            Write::Retire { slot } => {
                for g in &mut self.generations {
                    if &g.slot() == slot {
                        g.live = false;
                    }
                }
            }
            Write::Prune { scope } => self.generations.retain(|g| &g.scope != scope),
        }
        self.writes.push(write);
    }
}

const CONSTRAINTS: &[&str] = &[
    "CREATE CONSTRAINT memory_slot IF NOT EXISTS FOR (m:Memory) REQUIRE m.slot IS UNIQUE",
    "CREATE CONSTRAINT scope_name IF NOT EXISTS FOR (s:Scope) REQUIRE s.name IS UNIQUE",
    "CREATE CONSTRAINT subject_name IF NOT EXISTS FOR (t:Subject) REQUIRE t.name IS UNIQUE",
];

/// Refuse a database that holds a label neither relate-semantic nor relate-flow writes.
/// `Plan`, `Step`, and `Attempt` belong to relate-flow and may share this directory.
pub fn guard(conn: &mut Connection) -> Result<(), MemoryError> {
    let foreign = count(
        conn,
        "MATCH (n) WHERE NOT (n:Plan OR n:Step OR n:Attempt OR n:Memory OR n:Scope OR n:Subject) \
         RETURN count(n) AS c",
    )?;
    if foreign > 0 {
        return Err(MemoryError::refusal(
            "foreign_store",
            "the store holds a label that relate-semantic and relate-flow do not write",
            "open an empty directory, or a store one of those tools created, with -d PATH",
        ));
    }
    Ok(())
}

/// The unique constraints that make two racing writes to one key collide.
/// Schema statements run outside the command's transaction.
pub fn ensure_schema(conn: &mut Connection) -> Result<(), MemoryError> {
    for statement in CONSTRAINTS {
        query(conn, statement, BTreeMap::new())?;
    }
    Ok(())
}

pub fn load(conn: &mut Connection) -> Result<State, MemoryError> {
    let rows = query(
        conn,
        "MATCH (m:Memory) RETURN m.key, m.scope, m.generation, m.claim, m.because, m.about, \
         m.agent, m.at, m.forgotten, m.live",
        BTreeMap::new(),
    )?;
    let mut state = State::default();
    for row in &rows.rows {
        state.generations.push(Generation {
            key: req_string(&row[0], "memory.key")?,
            scope: req_string(&row[1], "memory.scope")?,
            generation: req_int(&row[2], "memory.generation")? as u64,
            claim: opt_string(&row[3]),
            because: req_string(&row[4], "memory.because")?,
            about: string_list(&row[5]),
            agent: req_string(&row[6], "memory.agent")?,
            at: req_string(&row[7], "memory.at")?,
            forgotten: opt_bool(&row[8]),
            live: opt_bool(&row[9]),
        });
    }
    state
        .generations
        .sort_by(|a, b| (&a.scope, &a.key, a.generation).cmp(&(&b.scope, &b.key, b.generation)));
    Ok(state)
}

pub fn apply(conn: &mut Connection, writes: &[Write]) -> Result<(), MemoryError> {
    for write in writes {
        match write {
            Write::Create {
                generation,
                supersedes,
            } => create(conn, generation, supersedes.as_deref())?,
            Write::Retire { slot } => {
                query(
                    conn,
                    "MATCH (m:Memory {slot: $slot}) SET m.live = false",
                    one("slot", v_str(slot)),
                )?;
            }
            Write::Prune { scope } => {
                query(
                    conn,
                    "MATCH (m:Memory {scope: $scope}) DETACH DELETE m",
                    one("scope", v_str(scope)),
                )?;
                query(
                    conn,
                    "MATCH (s:Scope {name: $scope}) DETACH DELETE s",
                    one("scope", v_str(scope)),
                )?;
                query(
                    conn,
                    "MATCH (t:Subject) OPTIONAL MATCH (t)<-[r:ABOUT]-() \
                     WITH t, count(r) AS uses WHERE uses = 0 DELETE t",
                    BTreeMap::new(),
                )?;
            }
        }
    }
    Ok(())
}

fn create(
    conn: &mut Connection,
    g: &Generation,
    supersedes: Option<&str>,
) -> Result<(), MemoryError> {
    let claim = g.claim.as_deref().map(v_str).unwrap_or(Value::Null);
    query(
        conn,
        "MERGE (s:Scope {name: $scope}) \
         CREATE (m:Memory {slot: $slot, key: $key, scope: $scope, generation: $generation, \
           claim: $claim, because: $because, about: $about, agent: $agent, at: $at, \
           forgotten: $forgotten, live: $live}) \
         CREATE (m)-[:IN]->(s)",
        params(&[
            ("slot", v_str(&g.slot())),
            ("key", v_str(&g.key)),
            ("scope", v_str(&g.scope)),
            ("generation", Value::Int(g.generation as i64)),
            ("claim", claim),
            ("because", v_str(&g.because)),
            ("about", v_list(&g.about)),
            ("agent", v_str(&g.agent)),
            ("at", v_str(&g.at)),
            ("forgotten", Value::Bool(g.forgotten)),
            ("live", Value::Bool(g.live)),
        ]),
    )?;
    for subject in &g.about {
        query(
            conn,
            "MATCH (m:Memory {slot: $slot}) MERGE (t:Subject {name: $name}) CREATE (m)-[:ABOUT]->(t)",
            params(&[("slot", v_str(&g.slot())), ("name", v_str(subject))]),
        )?;
    }
    if let Some(old) = supersedes {
        query(
            conn,
            "MATCH (n:Memory {slot: $new}), (o:Memory {slot: $old}) CREATE (n)-[:SUPERSEDES]->(o)",
            params(&[("new", v_str(&g.slot())), ("old", v_str(old))]),
        )?;
    }
    Ok(())
}

/// A race at statement time or at commit is the same thing to the caller: run
/// the command again.
pub fn is_race(err: &RelateError) -> bool {
    matches!(
        err,
        RelateError::Conflict | RelateError::UniquenessViolation { .. }
    )
}

pub fn race() -> MemoryError {
    MemoryError::conflict(
        "another write to the same memory committed first",
        "run the same command again",
    )
}

fn count(conn: &mut Connection, cypher: &str) -> Result<i64, MemoryError> {
    let result = query(conn, cypher, BTreeMap::new())?;
    Ok(result
        .rows
        .first()
        .and_then(|row| row.first())
        .and_then(opt_int)
        .unwrap_or(0))
}

fn query(
    conn: &mut Connection,
    cypher: &str,
    params: BTreeMap<String, Value>,
) -> Result<relate::QueryResult, MemoryError> {
    conn.query_with_params(cypher, params).map_err(|err| {
        if is_race(&err) {
            race()
        } else {
            MemoryError::store(err.to_string())
        }
    })
}

fn params(pairs: &[(&str, Value)]) -> BTreeMap<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.clone()))
        .collect()
}

fn one(key: &str, value: Value) -> BTreeMap<String, Value> {
    params(&[(key, value)])
}

fn v_str(text: &str) -> Value {
    Value::String(Arc::from(text))
}

fn v_list(items: &[String]) -> Value {
    Value::List(Arc::new(items.iter().map(|s| v_str(s)).collect()))
}

fn opt_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.to_string()),
        _ => None,
    }
}

fn req_string(value: &Value, what: &str) -> Result<String, MemoryError> {
    opt_string(value).ok_or_else(|| MemoryError::store(format!("missing {what}")))
}

fn opt_int(value: &Value) -> Option<i64> {
    match value {
        Value::Int(n) => Some(*n),
        _ => None,
    }
}

fn req_int(value: &Value, what: &str) -> Result<i64, MemoryError> {
    opt_int(value).ok_or_else(|| MemoryError::store(format!("missing {what}")))
}

fn opt_bool(value: &Value) -> bool {
    matches!(value, Value::Bool(true))
}

fn string_list(value: &Value) -> Vec<String> {
    match value {
        Value::List(items) => items.iter().filter_map(opt_string).collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readers_tolerate_missing_values() {
        assert_eq!(opt_string(&Value::Null), None);
        assert!(req_string(&Value::Null, "x").is_err());
        assert!(req_int(&Value::Null, "x").is_err());
        assert!(!opt_bool(&Value::Null));
        assert!(string_list(&Value::Null).is_empty());
        assert!(is_race(&RelateError::Conflict));
    }
}
