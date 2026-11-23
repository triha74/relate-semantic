use serde_json::{json, Value};

/// Every refusal the library can return. The CLI prints `to_json()` and exits
/// with [`MemoryError::exit_code`].
#[derive(Debug, Clone)]
pub struct MemoryError {
    pub code: &'static str,
    pub message: String,
    pub fix: String,
    pub retryable: bool,
    pub errors: Vec<Problem>,
    pub exit: u8,
}

/// One problem in a refusal that found several. `field` names the flag at fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub code: &'static str,
    pub message: String,
    pub fix: String,
    pub field: Option<String>,
}

impl MemoryError {
    pub fn usage(message: impl Into<String>, fix: impl Into<String>) -> Self {
        Self::new("usage", message, fix, false, 2)
    }

    pub fn refusal(code: &'static str, message: impl Into<String>, fix: impl Into<String>) -> Self {
        Self::new(code, message, fix, false, 4)
    }

    pub fn conflict(message: impl Into<String>, fix: impl Into<String>) -> Self {
        Self::new("conflict", message, fix, true, 5)
    }

    pub fn store(message: impl Into<String>) -> Self {
        Self::new(
            "store",
            message,
            "check the database path and retry",
            false,
            1,
        )
    }

    pub fn busy() -> Self {
        Self::new(
            "store_busy",
            "the store is locked by another process",
            "run the same command again",
            true,
            5,
        )
    }

    /// The first problem gives `code` and `message`. `fix` is the corrected
    /// command, which repairs every problem the agent does not have to rewrite.
    pub fn problems(problems: Vec<Problem>, fix: impl Into<String>) -> Self {
        let first = problems.first().cloned().expect("at least one problem");
        let mut err = Self::new(first.code, first.message, fix, false, 4);
        err.errors = problems;
        err
    }

    fn new(
        code: &'static str,
        message: impl Into<String>,
        fix: impl Into<String>,
        retryable: bool,
        exit: u8,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            fix: fix.into(),
            retryable,
            errors: Vec::new(),
            exit,
        }
    }

    pub fn exit_code(&self) -> i32 {
        i32::from(self.exit)
    }

    pub fn to_json(&self) -> Value {
        let mut body = json!({
            "code": self.code,
            "message": self.message,
            "fix": self.fix,
            "retryable": self.retryable,
        });
        if !self.errors.is_empty() {
            body["errors"] = Value::Array(
                self.errors
                    .iter()
                    .map(|p| {
                        let mut entry = json!({
                            "code": p.code,
                            "message": p.message,
                            "fix": p.fix,
                        });
                        if let Some(field) = &p.field {
                            entry["field"] = json!(field);
                        }
                        entry
                    })
                    .collect(),
            );
        }
        json!({ "error": body })
    }
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for MemoryError {}

/// Codes a client, the command matrix, and the docs can rely on. The last four
/// come from the `skill` verb, which never opens a store.
pub const ERROR_CODES: &[&str] = &[
    "usage",
    "store",
    "store_busy",
    "conflict",
    "foreign_store",
    "bad_slug",
    "empty_field",
    "too_long",
    "duplicate_value",
    "key_taken",
    "no_memory",
    "unknown_agent",
    "no_agent",
    "foreign_skill",
    "skill_drift",
    "skill_io",
];

/// Exit status for each code, in the same order as [`ERROR_CODES`].
pub fn exit_for(code: &str) -> Option<u8> {
    Some(match code {
        "usage" => 2,
        "store" | "skill_io" => 1,
        "store_busy" | "conflict" => 5,
        "foreign_store" | "bad_slug" | "empty_field" | "too_long" | "duplicate_value"
        | "key_taken" | "no_memory" | "unknown_agent" | "no_agent" | "foreign_skill"
        | "skill_drift" => 4,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_has_an_exit() {
        for code in ERROR_CODES {
            assert!(exit_for(code).is_some(), "{code}");
        }
        assert_eq!(exit_for("nope"), None);
    }

    #[test]
    fn constructors_set_the_exit_and_retryable() {
        assert_eq!(MemoryError::usage("m", "f").exit_code(), 2);
        assert_eq!(MemoryError::refusal("no_memory", "m", "f").exit_code(), 4);
        let conflict = MemoryError::conflict("m", "f");
        assert_eq!(conflict.exit_code(), 5);
        assert!(conflict.retryable);
        assert_eq!(MemoryError::store("m").exit_code(), 1);
        let busy = MemoryError::busy();
        assert_eq!(busy.code, "store_busy");
        assert!(busy.retryable);
        assert_eq!(
            format!("{busy}"),
            "store_busy: the store is locked by another process"
        );
    }

    #[test]
    fn problems_keep_the_first_code_and_list_every_field() {
        let err = MemoryError::problems(
            vec![
                Problem {
                    code: "bad_slug",
                    message: "key 'A' is not a slug".into(),
                    fix: "pass --key a".into(),
                    field: Some("key".into()),
                },
                Problem {
                    code: "empty_field",
                    message: "--claim is blank".into(),
                    fix: "write the fact in --claim".into(),
                    field: None,
                },
            ],
            "relate-semantic remember a --claim TEXT",
        );
        let body = err.to_json();
        assert_eq!(body["error"]["code"], "bad_slug");
        assert_eq!(
            body["error"]["fix"],
            "relate-semantic remember a --claim TEXT"
        );
        assert_eq!(body["error"]["errors"][0]["field"], "key");
        assert!(body["error"]["errors"][1].get("field").is_none());
    }
}
