//! Relate Semantic library. Every rule in the product design is applied here.
//! The CLI is a stateless caller.

mod clock;
mod error;
pub mod fix;
mod limits;
mod model;
mod ops;
mod recall;
mod slug;
mod store;
mod validate;

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use relate::{Database, Error as RelateError};
use serde_json::Value;

pub use clock::{parse_stamp, stamp, Clock, FixedClock, ManualClock, SystemClock};
pub use error::{exit_for, MemoryError, Problem, ERROR_CODES};
pub use limits::{DEFAULT_LIMIT, LOG_CAP, MAX_LIMIT, MAX_TEXT};
pub use model::Generation;
pub use ops::{Correct, Forget, Remember};
pub use recall::{decide, Fresh, Recall};
pub use slug::{is_slug, suggest};

/// The five answers `recall` can give, in decision order.
pub const RECALL_ACTIONS: &[&str] = &["conflict", "narrow", "stale", "use", "none"];

/// One embedded store. Cheap to clone. Connections are opened per command.
#[derive(Clone)]
pub struct Memory {
    db: Arc<Database>,
    clock: Arc<dyn Clock>,
}

impl Memory {
    /// Open or create the store at `uri`. `:memory:` is a store for tests.
    pub fn open(uri: &str) -> Result<Self, MemoryError> {
        Ok(Self {
            db: Arc::new(open_database(uri)?),
            clock: Arc::new(SystemClock),
        })
    }

    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    pub fn remember(&self, spec: Remember) -> Result<Value, MemoryError> {
        let now = self.now();
        self.write(move |state| ops::remember(state, spec, now))
    }

    pub fn correct(&self, spec: Correct) -> Result<Value, MemoryError> {
        let now = self.now();
        self.write(move |state| ops::correct(state, spec, now))
    }

    pub fn forget(&self, spec: Forget) -> Result<Value, MemoryError> {
        let now = self.now();
        self.write(move |state| ops::forget(state, spec, now))
    }

    pub fn recall(&self, query: Recall) -> Result<Value, MemoryError> {
        let now = self.now();
        self.read(move |state| ops::recall(state, query, now))
    }

    pub fn show(&self, key: &str) -> Result<Value, MemoryError> {
        self.read(|state| ops::show(state, key))
    }

    pub fn log(&self, key: &str, scope: Option<&str>) -> Result<Value, MemoryError> {
        self.read(|state| ops::log(state, key, scope))
    }

    pub fn prune(&self, scope: &str) -> Result<Value, MemoryError> {
        self.write(|state| ops::prune(state, scope))
    }

    /// Every scope with its number of live keys and generations, in name order.
    pub fn scopes(&self) -> Result<Value, MemoryError> {
        self.read(|state| Ok(ops::scopes(state)))
    }

    fn now(&self) -> DateTime<Utc> {
        self.clock.now()
    }

    fn read<F>(&self, f: F) -> Result<Value, MemoryError>
    where
        F: FnOnce(&store::State) -> Result<Value, MemoryError>,
    {
        let mut conn = self.db.connect().map_err(store_err)?;
        store::guard(&mut conn)?;
        conn.begin().map_err(store_err)?;
        let result = store::load(&mut conn).and_then(|state| f(&state));
        conn.rollback().ok();
        result
    }

    fn write<F>(&self, f: F) -> Result<Value, MemoryError>
    where
        F: FnOnce(&mut store::State) -> Result<Value, MemoryError>,
    {
        let mut conn = self.db.connect().map_err(store_err)?;
        store::guard(&mut conn)?;
        store::ensure_schema(&mut conn)?;
        conn.begin().map_err(store_err)?;
        let outcome = store::load(&mut conn).and_then(|mut state| {
            let value = f(&mut state)?;
            store::apply(&mut conn, &state.writes)?;
            Ok(value)
        });
        match outcome {
            Ok(value) => match conn.commit() {
                Ok(_) => Ok(value),
                Err(err) if store::is_race(&err) => Err(store::race()),
                Err(err) => Err(store_err(err)),
            },
            Err(err) => {
                conn.rollback().ok();
                Err(err)
            }
        }
    }
}

fn open_database(uri: &str) -> Result<Database, MemoryError> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match Database::open(uri) {
            Ok(db) => return Ok(db),
            Err(RelateError::Locked(_)) => {
                if Instant::now() >= deadline {
                    return Err(MemoryError::busy());
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(err) => return Err(store_err(err)),
        }
    }
}

fn store_err(err: RelateError) -> MemoryError {
    if matches!(err, RelateError::Locked(_)) {
        MemoryError::busy()
    } else {
        MemoryError::store(err.to_string())
    }
}
