#![allow(dead_code)]

pub mod op;

use std::sync::Arc;
use std::time::Duration;

use relate_semantic_core::{
    parse_stamp, Correct, FixedClock, Forget, Fresh, ManualClock, Memory, Recall, Remember,
    DEFAULT_LIMIT,
};

pub const T0: &str = "2026-11-10T18:00:00Z";
pub const PRICE: &str = "Under NW-2024, widgets are $100, with 10% off a widget line over $5000.";
pub const PRICE_2025: &str =
    "Under NW-2025, widgets are $110, with 10% off a widget line over $5000.";

pub fn open() -> Memory {
    open_at(":memory:")
}

pub fn open_at(uri: &str) -> Memory {
    Memory::open(uri)
        .unwrap()
        .with_clock(Arc::new(FixedClock(parse_stamp(T0).unwrap())))
}

/// A store whose clock the test moves.
pub fn open_manual() -> (Memory, Arc<ManualClock>) {
    let clock = Arc::new(ManualClock::new(parse_stamp(T0).unwrap()));
    let memory = Memory::open(":memory:").unwrap().with_clock(clock.clone());
    (memory, clock)
}

pub fn v(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

pub fn remember(key: &str, scope: &str, claim: &str, about: &[&str]) -> Remember {
    Remember {
        key: key.into(),
        scope: scope.into(),
        claim: claim.into(),
        because: "Read from the agreement while checking INV-1042.".into(),
        about: v(about),
        agent: "invoices".into(),
    }
}

pub fn correct(key: &str, scope: &str, claim: &str, about: Option<&[&str]>) -> Correct {
    Correct {
        key: key.into(),
        scope: scope.into(),
        claim: claim.into(),
        because: "NW-2025 replaced NW-2024 on 2026-11-01.".into(),
        about: about.map(v),
        agent: "invoices".into(),
    }
}

pub fn forget(key: &str, scope: &str) -> Forget {
    Forget {
        key: key.into(),
        scope: scope.into(),
        because: "The case override no longer applies.".into(),
        agent: "invoices".into(),
    }
}

pub fn recall(about: &[&str], scopes: &[&str]) -> Recall {
    Recall {
        about: v(about),
        keys: vec![],
        scopes: v(scopes),
        fresh: None,
        limit: DEFAULT_LIMIT,
    }
}

pub fn recall_keys(keys: &[&str], scopes: &[&str]) -> Recall {
    Recall {
        about: vec![],
        keys: v(keys),
        scopes: v(scopes),
        fresh: None,
        limit: DEFAULT_LIMIT,
    }
}

pub fn fresh(days: u64) -> Option<Fresh> {
    Some(Fresh {
        duration: Duration::from_secs(days * 86_400),
        label: format!("{days}d"),
    })
}

pub fn days(n: u64) -> Duration {
    Duration::from_secs(n * 86_400)
}
