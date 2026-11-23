use std::io::{self, IsTerminal};

use serde_json::Value;

#[cfg(test)]
static FORCE_TABLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn emit(value: &Value) {
    println!("{}", serde_json::to_string(value).expect("json"));
}

pub fn json_mode(force: bool) -> bool {
    #[cfg(test)]
    if FORCE_TABLE.load(std::sync::atomic::Ordering::Relaxed) && !force {
        return false;
    }
    force || !io::stdout().is_terminal()
}

pub fn log_table(value: &Value) -> String {
    let mut out = format!(
        "{:<11} {:<16} {:<12} {:<21} {}\n",
        "GENERATION", "SCOPE", "AGENT", "AT", "CLAIM"
    );
    for row in value["generations"].as_array().into_iter().flatten() {
        let generation = row["generation"].as_u64().unwrap_or(0);
        let scope = row["scope"].as_str().unwrap_or("");
        let agent = row["agent"].as_str().unwrap_or("");
        let at = row["at"].as_str().unwrap_or("");
        let claim = if row["forgotten"] == Value::Bool(true) {
            "(forgotten)".to_string()
        } else {
            clip(row["claim"].as_str().unwrap_or(""))
        };
        out.push_str(&format!(
            "{generation:<11} {scope:<16} {agent:<12} {at:<21} {claim}\n"
        ));
    }
    out
}

fn clip(text: &str) -> String {
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(60).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

pub fn print_log(value: &Value, as_json: bool) {
    if json_mode(as_json) {
        emit(value);
    } else {
        print!("{}", log_table(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn log_table_lists_generations_and_marks_tombstones() {
        let text = log_table(&json!({
            "generations": [
                {"generation": 2, "scope": "org", "agent": "invoices", "at": "2026-11-17T17:00:00Z",
                 "claim": null, "forgotten": true},
                {"generation": 1, "scope": "org", "agent": "invoices", "at": "2026-11-10T18:00:00Z",
                 "claim": "x".repeat(70), "forgotten": false}
            ]
        }));
        assert!(text.starts_with("GENERATION"));
        assert!(text.contains("(forgotten)"));
        assert!(text.contains('…'));
        assert!(text.contains("invoices"));
    }

    #[test]
    fn print_log_can_emit_the_table() {
        FORCE_TABLE.store(true, std::sync::atomic::Ordering::Relaxed);
        print_log(&json!({"generations": []}), false);
        FORCE_TABLE.store(false, std::sync::atomic::Ordering::Relaxed);
        print_log(&json!({"generations": []}), true);
    }
}
