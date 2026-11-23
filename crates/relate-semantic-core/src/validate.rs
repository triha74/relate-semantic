//! Collect every problem in one call, so a single refusal lists them all and
//! the corrected command repairs every one the agent does not have to rewrite.

use crate::error::{MemoryError, Problem};
use crate::limits::{length, MAX_TEXT};
use crate::slug::{is_slug, suggest};

#[derive(Default)]
pub struct Check {
    problems: Vec<Problem>,
}

impl Check {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check one slug. Returns the value to put in the corrected command.
    /// `flag` is `None` for the positional key.
    pub fn slug(
        &mut self,
        field: &str,
        flag: Option<&str>,
        value: &str,
        placeholder: &str,
    ) -> String {
        if is_slug(value) {
            return value.to_string();
        }
        let named = match flag {
            Some(flag) => format!("--{flag} '{value}'"),
            None => format!("{field} '{value}'"),
        };
        let suggestion = suggest(value);
        let fix = match (&suggestion, flag) {
            (Some(s), Some(flag)) => format!("pass --{flag} {s}"),
            (Some(s), None) => format!("use {s} as the {field}"),
            (None, _) => format!(
                "name the {field} with a lowercase letter, then lowercase letters, digits, and hyphens"
            ),
        };
        self.problems.push(Problem {
            code: "bad_slug",
            message: format!("{named} is not a slug"),
            fix,
            field: Some(field.to_string()),
        });
        suggestion.unwrap_or_else(|| placeholder.to_string())
    }

    /// Check a repeatable slug flag. Repeats are a problem and are dropped from
    /// the corrected list.
    pub fn slugs(
        &mut self,
        field: &str,
        flag: &str,
        values: &[String],
        placeholder: &str,
    ) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut seen: Vec<&str> = Vec::new();
        for value in values {
            if seen.contains(&value.as_str()) {
                self.problems.push(Problem {
                    code: "duplicate_value",
                    message: format!("--{flag} {value} is repeated"),
                    fix: format!("pass each --{flag} once"),
                    field: Some(field.to_string()),
                });
                continue;
            }
            seen.push(value);
            let fixed = self.slug(field, Some(flag), value, placeholder);
            if !out.contains(&fixed) || fixed == placeholder {
                out.push(fixed);
            }
        }
        out
    }

    /// A claim or a source: not blank, and at most [`MAX_TEXT`] characters.
    pub fn text(&mut self, field: &str, value: &str) {
        let flag = format!("--{field}");
        if value.trim().is_empty() {
            self.problems.push(Problem {
                code: "empty_field",
                message: format!("{flag} is blank"),
                fix: match field {
                    "claim" => "write the fact in --claim".to_string(),
                    _ => "say in --because where the claim came from".to_string(),
                },
                field: Some(field.to_string()),
            });
            return;
        }
        let chars = length(value);
        if chars > MAX_TEXT {
            self.problems.push(Problem {
                code: "too_long",
                message: format!("{flag} is {chars} characters and the limit is {MAX_TEXT}"),
                fix: match field {
                    "claim" => "write only the fact in --claim, and put the source in --because"
                        .to_string(),
                    _ => "shorten --because to where the claim came from".to_string(),
                },
                field: Some(field.to_string()),
            });
        }
    }

    /// `remember` needs at least one subject.
    pub fn required(&mut self, field: &str, flag: &str, values: &[String], verb: &str) {
        if values.is_empty() {
            self.problems.push(Problem {
                code: "empty_field",
                message: format!("{verb} needs at least one --{flag}"),
                fix: format!("pass --{flag} SUBJ for each subject the claim is about"),
                field: Some(field.to_string()),
            });
        }
    }

    pub fn finish(self, fix: String) -> Result<(), MemoryError> {
        if self.problems.is_empty() {
            Ok(())
        } else {
            Err(MemoryError::problems(self.problems, fix))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_clean_call_passes() {
        let mut check = Check::new();
        assert_eq!(
            check.slug("key", None, "widget-price", "KEY"),
            "widget-price"
        );
        assert_eq!(
            check.slugs("about", "about", &v(&["a", "b"]), "SUBJ"),
            v(&["a", "b"])
        );
        check.text("claim", "fact");
        check.required("about", "about", &v(&["a"]), "remember");
        assert!(check.finish("fix".into()).is_ok());
    }

    #[test]
    fn every_problem_is_listed() {
        let mut check = Check::new();
        assert_eq!(
            check.slug("key", None, "Widget Price", "KEY"),
            "widget-price"
        );
        assert_eq!(check.slug("scope", Some("scope"), "2024", "SCOPE"), "SCOPE");
        assert_eq!(
            check.slugs("about", "about", &v(&["Nw", "nw", "Nw"]), "SUBJ"),
            v(&["nw"])
        );
        check.text("claim", "  ");
        check.text("because", &"x".repeat(601));
        check.required("about", "about", &[], "remember");
        let err = check.finish("the fix".into()).unwrap_err();
        let codes: Vec<&str> = err.errors.iter().map(|p| p.code).collect();
        assert_eq!(
            codes,
            vec![
                "bad_slug",
                "bad_slug",
                "bad_slug",
                "duplicate_value",
                "empty_field",
                "too_long",
                "empty_field"
            ]
        );
        assert_eq!(err.code, "bad_slug");
        assert_eq!(err.fix, "the fix");
        assert_eq!(err.errors[0].fix, "use widget-price as the key");
        assert!(err.errors[1].fix.starts_with("name the scope"));
        assert_eq!(err.errors[2].fix, "pass --about nw");
        assert!(err.errors[5].message.contains("601 characters"));
    }

    #[test]
    fn six_hundred_characters_is_admitted() {
        let mut check = Check::new();
        check.text("claim", &"é".repeat(600));
        assert!(check.finish(String::new()).is_ok());
        let mut check = Check::new();
        check.text("claim", &"é".repeat(601));
        assert_eq!(check.finish(String::new()).unwrap_err().code, "too_long");
    }
}
