#![allow(dead_code)]

use std::cell::RefCell;

use assert_cmd::Command;
use serde_json::{json, Value};
use tempfile::TempDir;

pub const T0: &str = "2026-11-10T18:00:00Z";
pub const PRICE: &str = "Under NW-2024, widgets are $100, with 10% off a widget line over $5000.";
pub const PRICE_SRC: &str = "Read from the agreement while checking INV-1042.";
pub const PRICE_2025: &str =
    "Under NW-2025, widgets are $110, with 10% off a widget line over $5000.";

/// One store in a temporary directory. Every command gets `-d`, a fixed
/// `RELATE_SEMANTIC_NOW`, and `RELATE_SEMANTIC_AGENT=invoices` unless the test
/// changes them.
pub struct Store {
    dir: TempDir,
    now: RefCell<String>,
    agent: RefCell<Option<String>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
            now: RefCell::new(T0.into()),
            agent: RefCell::new(Some("invoices".into())),
        }
    }

    pub fn path(&self) -> String {
        self.dir.path().join("db").to_str().unwrap().to_string()
    }

    pub fn exists(&self) -> bool {
        self.dir.path().join("db").exists()
    }

    pub fn set_now(&self, now: &str) {
        *self.now.borrow_mut() = now.into();
    }

    pub fn set_agent(&self, agent: Option<&str>) {
        *self.agent.borrow_mut() = agent.map(str::to_string);
    }

    pub fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = bin();
        cmd.args(args).arg("-d").arg(self.path());
        cmd.env("RELATE_SEMANTIC_NOW", self.now.borrow().as_str());
        cmd.env_remove("RELATE_SEMANTIC_DATABASE");
        match self.agent.borrow().as_deref() {
            Some(agent) => cmd.env("RELATE_SEMANTIC_AGENT", agent),
            None => cmd.env_remove("RELATE_SEMANTIC_AGENT"),
        };
        cmd
    }

    pub fn run(&self, args: &[&str]) -> Value {
        parse(&self.cmd(args).assert().success().get_output().stdout)
    }

    pub fn fail(&self, args: &[&str], code: i32) -> Value {
        parse(
            &self
                .cmd(args)
                .assert()
                .failure()
                .code(code)
                .get_output()
                .stdout,
        )
    }

    pub fn text(&self, args: &[&str]) -> String {
        String::from_utf8(
            self.cmd(args)
                .assert()
                .success()
                .get_output()
                .stdout
                .clone(),
        )
        .unwrap()
    }

    /// `widget-price` in `org`, about northwind, nw-2024, and discounts.
    pub fn price(&self) -> Value {
        self.run(&[
            "remember",
            "widget-price",
            "--claim",
            PRICE,
            "--about",
            "northwind",
            "--about",
            "nw-2024",
            "--about",
            "discounts",
            "--scope",
            "org",
            "--because",
            PRICE_SRC,
        ])
    }

    /// A one-subject memory with a generated claim.
    pub fn note(&self, key: &str, scope: &str, about: &[&str]) -> Value {
        let claim = format!("{key} in {scope}");
        let mut args = vec![
            "remember",
            key,
            "--claim",
            &claim,
            "--scope",
            scope,
            "--because",
            "test",
        ];
        for subject in about {
            args.push("--about");
            args.push(subject);
        }
        self.run(&args)
    }
}

impl Store {
    /// Move the clock forward, as `advance` does in the example scripts.
    pub fn advance(&self, duration: &str) {
        let now = chrono::DateTime::parse_from_rfc3339(&self.now.borrow()).unwrap();
        let step = humantime::parse_duration(duration).unwrap();
        let later = now + chrono::Duration::from_std(step).unwrap();
        self.set_now(&later.to_utc().format("%Y-%m-%dT%H:%M:%SZ").to_string());
    }
}

/// Every command of a scenario with its full output, for one snapshot.
pub struct Transcript<'a> {
    store: &'a Store,
    steps: Vec<Value>,
}

impl<'a> Transcript<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self {
            store,
            steps: Vec::new(),
        }
    }

    pub fn run(&mut self, args: &[&str]) -> Value {
        let out = self.store.run(args);
        self.record(args, &out);
        out
    }

    pub fn refused(&mut self, code: &str, args: &[&str]) -> Value {
        let out = self.store.fail(args, 4);
        assert_eq!(out["error"]["code"], code, "{out}");
        self.record(args, &out);
        out
    }

    fn record(&mut self, args: &[&str], out: &Value) {
        self.steps.push(json!({
            "now": self.store.now.borrow().as_str(),
            "agent": self.store.agent.borrow().clone(),
            "args": args,
            "out": out,
        }));
    }

    /// The last recall's action, which the script header names.
    pub fn last_action(&self) -> String {
        self.steps
            .iter()
            .rev()
            .find_map(|s| s["out"]["action"].as_str().map(str::to_string))
            .unwrap_or_default()
    }

    pub fn steps(&self) -> Value {
        Value::Array(self.steps.clone())
    }
}

pub fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_relate-semantic"))
}

pub fn parse(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes)
        .unwrap_or_else(|err| panic!("{err}: {}", String::from_utf8_lossy(bytes)))
}

pub fn redact(value: &Value) -> Value {
    let mut copy = value.clone();
    walk(&mut copy);
    copy
}

fn walk(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if key == "at" {
                    *child = json!("[time]");
                } else {
                    walk(child);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(walk),
        _ => {}
    }
}
