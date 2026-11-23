use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use clap::error::ErrorKind;
use clap::{Parser, Subcommand, ValueEnum};
use relate_semantic_core::{
    fix, Correct, FixedClock, Forget, Fresh, Memory, MemoryError, Recall, Remember, DEFAULT_LIMIT,
};
use serde_json::Value;

use crate::output::{self, emit};
use crate::skill;

/// Every verb and its synopsis. A usage error's `fix` is the synopsis of the
/// verb that was called. `docs/cli.md` repeats each line.
pub const SYNOPSES: &[(&str, &str)] = &[
    (
        "recall",
        "relate-semantic recall [--about SUBJ]... [--key KEY]... [--scope SCOPE]... [--fresh DURATION] [--limit N] [--output json|text]",
    ),
    (
        "remember",
        "relate-semantic remember KEY --claim TEXT --about SUBJ... --scope SCOPE --because TEXT [--agent NAME]",
    ),
    (
        "correct",
        "relate-semantic correct KEY --scope SCOPE --claim TEXT --because TEXT [--about SUBJ]... [--agent NAME]",
    ),
    (
        "forget",
        "relate-semantic forget KEY --scope SCOPE --because TEXT [--agent NAME]",
    ),
    ("show", "relate-semantic show KEY"),
    ("log", "relate-semantic log KEY [--scope SCOPE] [--json]"),
    ("prune", "relate-semantic prune --scope SCOPE"),
    (
        "skill",
        "relate-semantic skill install|remove|list|check [--agent NAME] [--force]",
    ),
];

const TOP_SYNOPSIS: &str =
    "relate-semantic [-d PATH] [recall|remember|correct|forget|show|log|prune|skill]";

const DEFAULT_DATABASE: &str = "./.relate-semantic";

#[derive(Parser)]
#[command(
    name = "relate-semantic",
    version,
    about = "Facts agents chose to keep. Recall before you act. Remember what outlives the case",
    after_help = "With no verb, prints the scopes in the store as JSON.\n\n  relate-semantic recall --about northwind --about discounts --scope case-inv-1042 --scope org\n  relate-semantic remember widget-price --claim TEXT --about northwind --scope org --because TEXT --agent invoices"
)]
struct Cli {
    /// Relate store. RELATE_SEMANTIC_DATABASE is the same value. The flag wins.
    #[arg(
        short = 'd',
        long = "database",
        global = true,
        env = "RELATE_SEMANTIC_DATABASE",
        default_value = DEFAULT_DATABASE
    )]
    database: String,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Json,
    Text,
}

#[derive(Subcommand)]
enum Command {
    /// Return the live claims about these subjects, or say why there are none.
    Recall {
        /// A subject the memory must carry. Repeat for more. Every one must match.
        #[arg(long)]
        about: Vec<String>,
        /// A key the memory must have. Repeat for more.
        #[arg(long)]
        key: Vec<String>,
        /// A scope to search. Repeat in the order that should win.
        #[arg(long)]
        scope: Vec<String>,
        /// A winning claim older than this is stale. 30d, 12h.
        #[arg(long)]
        fresh: Option<String>,
        /// The most claims a `use` returns. 1 to 20.
        #[arg(long, default_value_t = DEFAULT_LIMIT)]
        limit: usize,
        /// `json` is one object. `text` prints the answer as a prompt.
        #[arg(long, value_enum, default_value = "json")]
        output: Format,
    },
    /// Write the first live claim for a key in a scope.
    Remember {
        key: String,
        #[arg(long)]
        claim: String,
        #[arg(long, required = true)]
        about: Vec<String>,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        because: String,
        #[arg(long, env = "RELATE_SEMANTIC_AGENT")]
        agent: Option<String>,
    },
    /// Replace the live claim for a key in a scope.
    Correct {
        key: String,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        claim: String,
        #[arg(long)]
        because: String,
        /// Replaces the whole subject list. Omitted keeps it.
        #[arg(long)]
        about: Vec<String>,
        #[arg(long, env = "RELATE_SEMANTIC_AGENT")]
        agent: Option<String>,
    },
    /// Stop a key being live in a scope. The history stays.
    Forget {
        key: String,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        because: String,
        #[arg(long, env = "RELATE_SEMANTIC_AGENT")]
        agent: Option<String>,
    },
    /// Print the live claim for a key in every scope.
    Show { key: String },
    /// Print every generation of a key, newest first.
    Log {
        key: String,
        #[arg(long)]
        scope: Option<String>,
        /// JSON even when stdout is a terminal.
        #[arg(long)]
        json: bool,
    },
    /// Delete every generation in one scope.
    Prune {
        #[arg(long)]
        scope: String,
    },
    /// Install or inspect the agent skill.
    Skill {
        #[command(subcommand)]
        action: SkillCommand,
    },
}

#[derive(Subcommand)]
enum SkillCommand {
    Install {
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        force: bool,
    },
    Remove {
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        force: bool,
    },
    List {
        #[arg(long)]
        agent: Option<String>,
    },
    Check {
        #[arg(long)]
        agent: Option<String>,
    },
}

pub fn run() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    run_with(&args)
}

pub fn run_with(args: &[String]) -> i32 {
    match Cli::try_parse_from(args) {
        Ok(cli) => dispatch(cli),
        Err(err) => match err.kind() {
            ErrorKind::DisplayHelp
            | ErrorKind::DisplayVersion
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                let _ = err.print();
                0
            }
            _ => finish(Err(usage_error(&err.to_string(), args))),
        },
    }
}

/// Turn clap's text into the JSON envelope. The fix is the synopsis of the verb
/// that was called.
fn usage_error(text: &str, args: &[String]) -> MemoryError {
    let mut lines = text.lines().skip_while(|line| line.trim().is_empty());
    let first = lines
        .next()
        .map(|line| line.trim().trim_start_matches("error: ").to_string())
        .unwrap_or_else(|| "the flags are wrong".to_string());
    let detail: Vec<&str> = lines
        .take_while(|line| line.starts_with(' ') && !line.trim().is_empty())
        .map(str::trim)
        .collect();
    let message = if detail.is_empty() {
        first
    } else {
        format!("{} {}", first, detail.join(", "))
    };
    let fix = args
        .iter()
        .skip(1)
        .find_map(|arg| SYNOPSES.iter().find(|(verb, _)| verb == arg))
        .map(|(_, synopsis)| synopsis.to_string())
        .unwrap_or_else(|| TOP_SYNOPSIS.to_string());
    MemoryError::usage(message, fix)
}

fn dispatch(cli: Cli) -> i32 {
    let database = cli.database;
    let Some(command) = cli.command else {
        return finish(open_read(&database).and_then(|m| m.scopes()));
    };
    match command {
        Command::Recall {
            about,
            key,
            scope,
            fresh,
            limit,
            output,
        } => {
            let fresh = match parse_fresh(fresh) {
                Ok(fresh) => fresh,
                Err(mut err) => {
                    err.fix = format!("{} --fresh DURATION", fix::recall(&about, &key, &scope));
                    return finish(Err(err));
                }
            };
            let query = Recall {
                about,
                keys: key,
                scopes: scope,
                fresh,
                limit,
            };
            let answer = open_read(&database).and_then(|m| m.recall(query));
            if matches!(output, Format::Text) {
                if let Ok(value) = &answer {
                    print!("{}", crate::prompt::render(value));
                    return 0;
                }
            }
            finish(answer)
        }
        Command::Remember {
            key,
            claim,
            about,
            scope,
            because,
            agent,
        } => {
            let agent = match require_agent("remember", agent) {
                Ok(agent) => agent,
                Err(err) => return finish(Err(err)),
            };
            let spec = Remember {
                key,
                scope,
                claim,
                because,
                about,
                agent,
            };
            finish(open(&database).and_then(|m| m.remember(spec)))
        }
        Command::Correct {
            key,
            scope,
            claim,
            because,
            about,
            agent,
        } => {
            let agent = match require_agent("correct", agent) {
                Ok(agent) => agent,
                Err(err) => return finish(Err(err)),
            };
            let spec = Correct {
                key,
                scope,
                claim,
                because,
                about: if about.is_empty() { None } else { Some(about) },
                agent,
            };
            finish(open(&database).and_then(|m| m.correct(spec)))
        }
        Command::Forget {
            key,
            scope,
            because,
            agent,
        } => {
            let agent = match require_agent("forget", agent) {
                Ok(agent) => agent,
                Err(err) => return finish(Err(err)),
            };
            let spec = Forget {
                key,
                scope,
                because,
                agent,
            };
            finish(open(&database).and_then(|m| m.forget(spec)))
        }
        Command::Show { key } => finish(open_read(&database).and_then(|m| m.show(&key))),
        Command::Log { key, scope, json } => {
            match open_read(&database).and_then(|m| m.log(&key, scope.as_deref())) {
                Ok(value) => {
                    output::print_log(&value, json);
                    0
                }
                Err(err) => finish(Err(err)),
            }
        }
        Command::Prune { scope } => finish(open_read(&database).and_then(|m| m.prune(&scope))),
        Command::Skill { action } => match action {
            SkillCommand::Install { agent, force } => skill::install(agent.as_deref(), force),
            SkillCommand::Remove { agent, force } => skill::remove(agent.as_deref(), force),
            SkillCommand::List { agent } => skill::list(agent.as_deref()),
            SkillCommand::Check { agent } => skill::check(agent.as_deref()),
        },
    }
}

fn require_agent(verb: &str, agent: Option<String>) -> Result<String, MemoryError> {
    agent.filter(|a| !a.is_empty()).ok_or_else(|| {
        MemoryError::usage(
            format!("{verb} requires --agent"),
            "pass --agent NAME or set RELATE_SEMANTIC_AGENT",
        )
    })
}

fn parse_fresh(text: Option<String>) -> Result<Option<Fresh>, MemoryError> {
    let Some(text) = text else {
        return Ok(None);
    };
    let duration = humantime::parse_duration(&text).map_err(|_| {
        MemoryError::usage(
            format!("cannot parse --fresh duration '{text}'"),
            "pass --fresh as a duration such as 30d or 12h",
        )
    })?;
    Ok(Some(Fresh {
        duration,
        label: text,
    }))
}

/// A verb that only reads, or deletes, never creates the store. A missing
/// directory reads as an empty store.
fn open_read(path: &str) -> Result<Memory, MemoryError> {
    if path != ":memory:" && !Path::new(path).exists() {
        return open(":memory:");
    }
    open(path)
}

fn open(path: &str) -> Result<Memory, MemoryError> {
    let memory = Memory::open(path)?;
    match std::env::var("RELATE_SEMANTIC_NOW") {
        Ok(raw) if !raw.is_empty() => {
            let time = DateTime::parse_from_rfc3339(&raw).map_err(|_| {
                MemoryError::usage(
                    format!("RELATE_SEMANTIC_NOW '{raw}' is not RFC3339"),
                    "set RELATE_SEMANTIC_NOW to an RFC3339 UTC timestamp, or unset it",
                )
            })?;
            Ok(memory.with_clock(Arc::new(FixedClock(time.with_timezone(&Utc)))))
        }
        _ => Ok(memory),
    }
}

fn finish(result: Result<Value, MemoryError>) -> i32 {
    match result {
        Ok(value) => {
            emit(&value);
            0
        }
        Err(err) => {
            emit(&err.to_json());
            err.exit_code()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_fix_is_the_synopsis_of_the_called_verb() {
        let args: Vec<String> = ["relate-semantic", "remember", "k"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let err = usage_error(
            "error: the following required arguments were not provided:\n  --claim <CLAIM>\n  --scope <SCOPE>\n\nUsage: x",
            &args,
        );
        assert_eq!(err.code, "usage");
        assert_eq!(
            err.message,
            "the following required arguments were not provided: --claim <CLAIM>, --scope <SCOPE>"
        );
        assert!(err.fix.starts_with("relate-semantic remember KEY"));
        let err = usage_error("", &["relate-semantic".to_string(), "frob".to_string()]);
        assert_eq!(err.fix, TOP_SYNOPSIS);
        assert_eq!(err.message, "the flags are wrong");
    }

    #[test]
    fn every_subcommand_has_a_synopsis() {
        use clap::CommandFactory;
        let mut names: Vec<String> = Cli::command()
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .filter(|n| n != "help")
            .collect();
        names.sort();
        let mut verbs: Vec<String> = SYNOPSES.iter().map(|(v, _)| v.to_string()).collect();
        verbs.sort();
        assert_eq!(names, verbs);
    }
}
