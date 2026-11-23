//! `recall --output text`. The first line is always `Recall: ACTION …`, so a
//! harness can switch on it the way it switches on `action` in JSON.

use serde_json::Value;

pub fn render(value: &Value) -> String {
    let action = value["action"].as_str().unwrap_or("");
    match action {
        "use" => {
            let memories = list(&value["memories"]);
            let mut out = format!(
                "Recall: use {} {}.\nUse these claims as written. Do not re-derive them.\n",
                memories.len(),
                plural(memories.len(), "memory", "memories")
            );
            for memory in memories {
                out.push_str(&block(memory, false));
            }
            out
        }
        "stale" => {
            let stale = strings(&value["stale"]);
            let fresh = value["fresh"].as_str().unwrap_or("");
            let mut out = format!(
                "Recall: stale {}.\nOlder than {fresh}: {}. Check the source in each, then correct it, or recall without --fresh.\n",
                stale.join(", "),
                stale.join(", ")
            );
            for memory in list(&value["memories"]) {
                let key = memory["key"].as_str().unwrap_or("");
                out.push_str(&block(memory, stale.iter().any(|s| s == key)));
            }
            out
        }
        "none" => {
            let about = strings(&value["about"]);
            let keys = strings(&value["keys"]);
            let scopes = strings(&value["scopes"]);
            let mut what = Vec::new();
            if !about.is_empty() {
                what.push(format!("about {}", about.join(", ")));
            }
            if !keys.is_empty() {
                what.push(format!("for {}", keys.join(", ")));
            }
            let place = if scopes.is_empty() {
                "any scope".to_string()
            } else {
                scopes.join(", ")
            };
            format!(
                "Recall: none.\nNothing is remembered {} in {place}. Do the work, and remember a fact a later case would need.\n",
                what.join(" ")
            )
        }
        "narrow" => {
            let keys: Vec<String> = list(&value["keys"])
                .iter()
                .map(|k| k["key"].as_str().unwrap_or("").to_string())
                .collect();
            format!(
                "Recall: narrow {} keys matched.\nThe limit is {}. Recall again with --key or another --about: {}\nKeys: {}\n",
                value["matched"],
                value["limit"],
                value["fix"].as_str().unwrap_or(""),
                keys.join(", ")
            )
        }
        "conflict" => format!(
            "Recall: conflict on {}.\nIt is live in {}. Recall again with --scope in the order that should win: {}\n",
            value["key"].as_str().unwrap_or(""),
            strings(&value["scopes"]).join(", "),
            value["fix"].as_str().unwrap_or("")
        ),
        _ => format!("{value}\n"),
    }
}

fn block(memory: &Value, stale: bool) -> String {
    format!(
        "\n{}{} ({}, generation {}, {}, by {})\n{}\nSource: {}\n",
        if stale { "[stale] " } else { "" },
        memory["key"].as_str().unwrap_or(""),
        memory["scope"].as_str().unwrap_or(""),
        memory["generation"],
        memory["at"].as_str().unwrap_or(""),
        memory["agent"].as_str().unwrap_or(""),
        memory["claim"].as_str().unwrap_or(""),
        memory["because"].as_str().unwrap_or(""),
    )
}

fn list(value: &Value) -> Vec<&Value> {
    value
        .as_array()
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}

fn strings(value: &Value) -> Vec<String> {
    list(value)
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect()
}

fn plural<'a>(n: usize, one: &'a str, many: &'a str) -> &'a str {
    if n == 1 {
        one
    } else {
        many
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn memory() -> Value {
        json!({"key": "widget-price", "scope": "org", "claim": "c", "because": "b",
               "about": ["a"], "agent": "invoices", "generation": 1, "at": "2026-11-10T18:00:00Z"})
    }

    #[test]
    fn every_action_starts_with_recall() {
        let answers = [
            json!({"action": "use", "memories": [memory()]}),
            json!({"action": "stale", "stale": ["widget-price"], "fresh": "30d", "memories": [memory()]}),
            json!({"action": "none", "about": ["a"], "keys": ["k"], "scopes": []}),
            json!({"action": "narrow", "matched": 9, "limit": 5, "keys": [{"key": "k"}], "fix": "relate-semantic recall --key k"}),
            json!({"action": "conflict", "key": "k", "scopes": ["a", "b"], "fix": "relate-semantic recall --key k --scope a --scope b"}),
        ];
        for answer in answers {
            let text = render(&answer);
            let expected = format!("Recall: {}", answer["action"].as_str().unwrap());
            assert!(text.starts_with(&expected), "{text}");
        }
        assert!(render(&answers_use_two()).contains("use 2 memories"));
        assert!(render(&json!({"action": "stale", "stale": ["widget-price"], "fresh": "30d", "memories": [memory()]})).contains("[stale] widget-price"));
        assert!(
            render(&json!({"action": "none", "about": [], "keys": [], "scopes": ["org"]}))
                .contains("in org")
        );
        assert_eq!(render(&json!({"x": 1})), "{\"x\":1}\n");
    }

    fn answers_use_two() -> Value {
        json!({"action": "use", "memories": [memory(), memory()]})
    }
}
