//! Racing writers on one store. Each thread opens its own connection, as two
//! processes would.

mod support;

use std::sync::{Arc, Barrier};
use std::thread;

use relate_semantic_core::{Memory, MemoryError};
use serde_json::Value;
use support::*;

fn race<F>(
    memory: &Memory,
    left: F,
    right: F,
) -> (Result<Value, MemoryError>, Result<Value, MemoryError>)
where
    F: FnOnce(&Memory) -> Result<Value, MemoryError> + Send + 'static,
{
    let barrier = Arc::new(Barrier::new(2));
    let spawn = |f: F| {
        let memory = memory.clone();
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            barrier.wait();
            f(&memory)
        })
    };
    let l = spawn(left);
    let r = spawn(right);
    (l.join().unwrap(), r.join().unwrap())
}

fn retry<F: Fn() -> Result<Value, MemoryError>>(f: F) -> Value {
    for _ in 0..8 {
        match f() {
            Ok(value) => return value,
            Err(err) if err.code == "conflict" && err.retryable => continue,
            Err(err) => panic!("{err:?}"),
        }
    }
    panic!("still conflicting after 8 tries");
}

fn live_generations(memory: &Memory, key: &str) -> Vec<u64> {
    memory.log(key, Some("org")).unwrap()["generations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|g| g["live"] == true)
        .map(|g| g["generation"].as_u64().unwrap())
        .collect()
}

#[test]
fn two_corrections_of_one_key_leave_one_live_generation() {
    for _ in 0..8 {
        let memory = open();
        memory
            .remember(remember("price", "org", PRICE, &["nw"]))
            .unwrap();
        let (left, right) = race(
            &memory,
            Box::new(|m: &Memory| m.correct(correct("price", "org", "left", None)))
                as Box<dyn FnOnce(&Memory) -> Result<Value, MemoryError> + Send>,
            Box::new(|m: &Memory| m.correct(correct("price", "org", "right", None))),
        );
        let wins = [&left, &right].iter().filter(|r| r.is_ok()).count();
        for result in [&left, &right] {
            if let Err(err) = result {
                assert_eq!(err.code, "conflict", "{err:?}");
                assert!(err.retryable);
                assert_eq!(err.exit_code(), 5);
                assert_eq!(err.fix, "run the same command again");
            }
        }
        assert!(wins >= 1, "{left:?} {right:?}");
        let live = live_generations(&memory, "price");
        assert_eq!(live.len(), 1, "{left:?} {right:?}");
        assert_eq!(live[0], 1 + wins as u64);
    }
}

#[test]
fn two_remembers_of_one_new_key_leave_one_generation() {
    for _ in 0..8 {
        let memory = open();
        memory
            .remember(remember("other", "org", "x", &["nw"]))
            .unwrap();
        let (left, right) = race(
            &memory,
            Box::new(|m: &Memory| m.remember(remember("price", "org", "left", &["nw"])))
                as Box<dyn FnOnce(&Memory) -> Result<Value, MemoryError> + Send>,
            Box::new(|m: &Memory| m.remember(remember("price", "org", "right", &["nw"]))),
        );
        let codes: Vec<&str> = [&left, &right]
            .iter()
            .filter_map(|r| r.as_ref().err().map(|e| e.code))
            .collect();
        assert_eq!(codes.len(), 1, "{left:?} {right:?}");
        assert!(matches!(codes[0], "conflict" | "key_taken"), "{codes:?}");
        assert_eq!(live_generations(&memory, "price"), vec![1]);
    }
}

#[test]
fn writes_to_different_keys_in_one_scope_both_commit() {
    for _ in 0..8 {
        let memory = open();
        memory
            .remember(remember("seed", "org", "x", &["nw"]))
            .unwrap();
        let (left, right) = race(
            &memory,
            Box::new(|m: &Memory| m.remember(remember("price", "org", "p", &["nw"])))
                as Box<dyn FnOnce(&Memory) -> Result<Value, MemoryError> + Send>,
            Box::new(|m: &Memory| m.remember(remember("terms", "org", "t", &["nw"]))),
        );
        assert!(left.is_ok(), "{left:?}");
        assert!(right.is_ok(), "{right:?}");
        let listing = memory.scopes().unwrap();
        assert_eq!(listing["scopes"][0]["live"], 3);
    }
}

#[test]
fn a_new_scope_race_settles_with_one_retry() {
    for _ in 0..8 {
        let memory = open();
        let (left, right) = race(
            &memory,
            Box::new(|m: &Memory| {
                Ok(retry(|| {
                    m.remember(remember("price", "case-7", "p", &["new-subject"]))
                }))
            }) as Box<dyn FnOnce(&Memory) -> Result<Value, MemoryError> + Send>,
            Box::new(|m: &Memory| {
                Ok(retry(|| {
                    m.remember(remember("terms", "case-7", "t", &["new-subject"]))
                }))
            }),
        );
        assert!(left.is_ok() && right.is_ok());
        let listing = memory.scopes().unwrap();
        assert_eq!(listing["scopes"][0]["live"], 2);
    }
}
