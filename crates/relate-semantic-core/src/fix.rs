//! The commands a `fix` names. Every value is a slug or a placeholder, so a
//! fix never needs shell quoting.

pub const TEXT: &str = "TEXT";
pub const SUBJ: &str = "SUBJ";
pub const SCOPE: &str = "SCOPE";
pub const KEY: &str = "KEY";

/// Placeholders a fix may contain. Anything else in a fix is a literal value.
pub const PLACEHOLDERS: &[&str] = &[TEXT, SUBJ, SCOPE, KEY, "DURATION"];

fn flags(out: &mut String, flag: &str, values: &[String]) {
    for value in values {
        out.push_str(&format!(" --{flag} {value}"));
    }
}

fn agent(out: &mut String, agent: Option<&str>) {
    if let Some(agent) = agent {
        out.push_str(&format!(" --agent {agent}"));
    }
}

pub fn remember(key: &str, about: &[String], scope: &str, by: Option<&str>) -> String {
    let mut out = format!("relate-semantic remember {key} --claim {TEXT}");
    if about.is_empty() {
        out.push_str(&format!(" --about {SUBJ}"));
    } else {
        flags(&mut out, "about", about);
    }
    out.push_str(&format!(" --scope {scope} --because {TEXT}"));
    agent(&mut out, by);
    out
}

pub fn correct(key: &str, scope: &str, about: Option<&[String]>, by: Option<&str>) -> String {
    let mut out =
        format!("relate-semantic correct {key} --scope {scope} --claim {TEXT} --because {TEXT}");
    if let Some(about) = about {
        flags(&mut out, "about", about);
    }
    agent(&mut out, by);
    out
}

pub fn forget(key: &str, scope: &str, by: Option<&str>) -> String {
    let mut out = format!("relate-semantic forget {key} --scope {scope} --because {TEXT}");
    agent(&mut out, by);
    out
}

pub fn recall(about: &[String], keys: &[String], scopes: &[String]) -> String {
    let mut out = "relate-semantic recall".to_string();
    flags(&mut out, "about", about);
    flags(&mut out, "key", keys);
    flags(&mut out, "scope", scopes);
    if about.is_empty() && keys.is_empty() {
        out.push_str(&format!(" --about {SUBJ}"));
    }
    out
}

pub fn log(key: &str, scope: Option<&str>) -> String {
    match scope {
        Some(scope) => format!("relate-semantic log {key} --scope {scope}"),
        None => format!("relate-semantic log {key}"),
    }
}

pub fn show(key: &str) -> String {
    format!("relate-semantic show {key}")
}

pub fn prune(scope: &str) -> String {
    format!("relate-semantic prune --scope {scope}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn commands_are_filled_in() {
        assert_eq!(
            remember("k", &v(&["a", "b"]), "org", None),
            "relate-semantic remember k --claim TEXT --about a --about b --scope org --because TEXT"
        );
        assert_eq!(
            remember("k", &[], "org", Some("bot")),
            "relate-semantic remember k --claim TEXT --about SUBJ --scope org --because TEXT --agent bot"
        );
        assert_eq!(
            correct("k", "org", Some(&v(&["a"])), None),
            "relate-semantic correct k --scope org --claim TEXT --because TEXT --about a"
        );
        assert_eq!(
            forget("k", "org", Some("bot")),
            "relate-semantic forget k --scope org --because TEXT --agent bot"
        );
        assert_eq!(
            recall(&[], &v(&["k"]), &v(&["a", "b"])),
            "relate-semantic recall --key k --scope a --scope b"
        );
        assert_eq!(recall(&[], &[], &[]), "relate-semantic recall --about SUBJ");
        assert_eq!(log("k", Some("org")), "relate-semantic log k --scope org");
        assert_eq!(log("k", None), "relate-semantic log k");
        assert_eq!(show("k"), "relate-semantic show k");
        assert_eq!(prune("org"), "relate-semantic prune --scope org");
    }
}
