use std::{path::Path, process::Command};

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    #[serde(rename = "headRefName")]
    pub head: String,
    #[serde(rename = "baseRefName")]
    pub base: String,
    #[serde(rename = "isDraft")]
    pub draft: bool,
    pub state: String,
    pub url: String,
}

#[derive(Default)]
pub struct GitSnapshot {
    pub repo: String,
    pub branch: String,
    pub status: String,
    pub files: String,
    pub commits: String,
    pub branches: String,
    pub stash: String,
    pub remotes: String,
}

pub enum JobKind {
    Overview,
    Git,
    PullRequests,
    Details(u64),
    Checks(u64),
    Raw(String),
}

pub enum JobResult {
    Overview {
        git: GitSnapshot,
        github: String,
        items: Vec<PullRequest>,
        pr_message: String,
    },
    Git(GitSnapshot),
    PullRequests {
        items: Vec<PullRequest>,
        message: String,
    },
    Details {
        number: u64,
        text: String,
    },
    Checks {
        number: u64,
        text: String,
    },
    Raw(String),
}

pub fn execute_job(kind: JobKind) -> JobResult {
    match kind {
        JobKind::Overview => {
            let (items, pr_message) = load_pull_requests();
            JobResult::Overview {
                git: load_git(),
                github: command_text("gh", &["pr", "status"]),
                items,
                pr_message,
            }
        }
        JobKind::Git => JobResult::Git(load_git()),
        JobKind::PullRequests => {
            let (items, message) = load_pull_requests();
            JobResult::PullRequests { items, message }
        }
        JobKind::Details(number) => {
            let text = match invoke(
                "gh",
                &[
                    "pr",
                    "view",
                    &number.to_string(),
                    "--json",
                    "number,title,state,author,headRefName,baseRefName,isDraft,url,body,reviewDecision,mergeStateStatus",
                ],
            ) {
                Ok(json) => format_pr_detail(&json),
                Err(error) => error,
            };
            JobResult::Details { number, text }
        }
        JobKind::Checks(number) => JobResult::Checks {
            number,
            text: command_text("gh", &["pr", "checks", &number.to_string()]),
        },
        JobKind::Raw(command) => JobResult::Raw(run_raw_command(&command)),
    }
}

fn load_git() -> GitSnapshot {
    let root = command_text("git", &["rev-parse", "--show-toplevel"]);
    let repo = Path::new(root.trim())
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repository")
        .to_string();
    GitSnapshot {
        repo,
        branch: command_text("git", &["symbolic-ref", "--short", "HEAD"]),
        status: command_text("git", &["status", "--short", "--branch"]),
        files: command_text("git", &["status", "--short"]),
        commits: command_text("git", &["log", "--oneline", "--decorate", "-n", "20"]),
        branches: command_text("git", &["branch", "-vv"]),
        stash: command_text("git", &["stash", "list"]),
        remotes: command_text("git", &["remote", "-v"]),
    }
}

fn load_pull_requests() -> (Vec<PullRequest>, String) {
    match invoke(
        "gh",
        &[
            "pr",
            "list",
            "--limit",
            "50",
            "--json",
            "number,title,state,headRefName,baseRefName,isDraft,url",
        ],
    ) {
        Ok(text) => match serde_json::from_str::<Vec<PullRequest>>(&text) {
            Ok(items) => {
                let message = format!("{} pull request(s)", items.len());
                (items, message)
            }
            Err(error) => (
                Vec::new(),
                format!("Could not read gh pr list response: {error}\n\n{text}"),
            ),
        },
        Err(message) => (Vec::new(), message),
    }
}

fn format_pr_detail(json: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(json) else {
        return json.to_string();
    };
    let field = |key: &str| value.get(key).and_then(Value::as_str).unwrap_or("—");
    let author = value
        .get("author")
        .and_then(|item| item.get("login"))
        .and_then(Value::as_str)
        .unwrap_or("—");
    let number = value
        .get("number")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    format!(
        "{}  #{}\n\n{} → {}\n\nState   {}\nReview  {}\nMerge   {}\nAuthor  {}\n\n{}\n\n{}",
        field("title"),
        number,
        field("headRefName"),
        field("baseRefName"),
        field("state"),
        field("reviewDecision"),
        field("mergeStateStatus"),
        author,
        field("url"),
        field("body")
    )
}

fn command_text(program: &str, args: &[&str]) -> String {
    match invoke(program, args) {
        Ok(text) if text.trim().is_empty() => "(no output)".into(),
        Ok(text) => text,
        Err(error) => error,
    }
}

fn invoke(program: &str, args: &[&str]) -> Result<String, String> {
    let result = Command::new(program).args(args).output().map_err(|error| {
        format!("Could not start {program}: {error}\nCheck that the CLI is available in PATH.")
    })?;
    let stdout = String::from_utf8_lossy(&result.stdout)
        .trim_end()
        .to_string();
    let stderr = String::from_utf8_lossy(&result.stderr)
        .trim_end()
        .to_string();
    if result.status.success() {
        Ok(match (stdout.is_empty(), stderr.is_empty()) {
            (false, false) => format!("{stdout}\n\n{stderr}"),
            (false, true) => stdout,
            (true, _) => stderr,
        })
    } else {
        let code = result
            .status
            .code()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "signal".into());
        let details = [stderr, stdout]
            .into_iter()
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        Err(format!(
            "{program} {} failed (exit {code})\n{details}",
            args.join(" ")
        ))
    }
}

fn run_raw_command(command: &str) -> String {
    let args = match split_args(command.trim()) {
        Ok(args) => args,
        Err(error) => return error,
    };
    let Some((program, rest)) = args.split_first() else {
        return "Enter a command beginning with git or gh".into();
    };
    if program != "git" && program != "gh" {
        return "Only git and gh commands are available. Example: gh pr checks 42".into();
    }
    match Command::new(program).args(rest).output() {
        Err(error) => format!("Could not start {program}: {error}"),
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout)
                .trim_end()
                .to_string();
            let stderr = String::from_utf8_lossy(&result.stderr)
                .trim_end()
                .to_string();
            let code = result
                .status
                .code()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "signal".into());
            format!(
                "$ {command}\nexit {code}\n\n{}",
                [stdout, stderr]
                    .into_iter()
                    .filter(|text| !text.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n\n")
            )
        }
    }
}

fn split_args(input: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut escape = false;
    let mut active = false;
    for ch in input.chars() {
        if escape {
            word.push(ch);
            escape = false;
            active = true;
        } else if ch == '\\' && quote != Some('\'') {
            escape = true;
            active = true;
        } else if let Some(q) = quote {
            if ch == q {
                quote = None
            } else {
                word.push(ch)
            }
            active = true;
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
            active = true;
        } else if ch.is_whitespace() {
            if active {
                args.push(std::mem::take(&mut word));
                active = false;
            }
        } else {
            word.push(ch);
            active = true;
        }
    }
    if escape {
        return Err("Command ends with an unfinished escape".into());
    }
    if quote.is_some() {
        return Err("Command has an unclosed quote".into());
    }
    if active {
        args.push(word);
    }
    Ok(args)
}
