use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use crossterm::event::KeyCode;

use crate::commands::{GitSnapshot, JobKind, JobResult, PullRequest, execute_job};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overview,
    Git,
    Branches,
    Commits,
    Stash,
    PullRequests,
    Checks,
    Output,
}

impl View {
    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "Status",
            Self::Git => "Files",
            Self::Branches => "Local branches",
            Self::Commits => "Commits",
            Self::Stash => "Stash",
            Self::PullRequests => "Pull requests",
            Self::Checks => "Checks",
            Self::Output => "Command output",
        }
    }
}

pub struct App {
    pub view: View,
    pub overview_github: String,
    pub git: GitSnapshot,
    pub pull_requests: Vec<PullRequest>,
    pub pr_message: String,
    pub selected_pr: usize,
    pub detail: String,
    pub detail_number: Option<u64>,
    pub checks: String,
    pub checks_number: Option<u64>,
    pub output: String,
    pub scroll: u16,
    pub loading: bool,
    pub input: Option<String>,
    pub notice: String,
    pub command_log: String,
    pending: Option<JobKind>,
    tx: Sender<JobResult>,
    rx: Receiver<JobResult>,
}

impl App {
    pub fn new(tx: Sender<JobResult>, rx: Receiver<JobResult>) -> Self {
        Self {
            view: View::Overview,
            overview_github: "Loading GitHub pull requests…".into(),
            git: GitSnapshot::default(),
            pull_requests: Vec::new(),
            pr_message: "Press p to load pull requests".into(),
            selected_pr: 0,
            detail: String::new(),
            detail_number: None,
            checks: String::new(),
            checks_number: None,
            output: "Press : to run a git or gh command".into(),
            scroll: 0,
            loading: false,
            input: None,
            notice: String::new(),
            command_log: "Starting repository scan…".into(),
            pending: None,
            tx,
            rx,
        }
    }

    pub fn selected_pr(&self) -> Option<&PullRequest> {
        self.pull_requests.get(self.selected_pr)
    }

    pub fn run(&mut self, kind: JobKind) {
        if self.loading {
            self.notice = "The next view is queued…".into();
            self.pending = Some(kind);
            return;
        }
        self.loading = true;
        self.notice.clear();
        self.scroll = 0;
        self.command_log = match &kind {
            JobKind::Overview | JobKind::Git => {
                "Running git status, branches, commits, stash…".into()
            }
            JobKind::PullRequests => "Running gh pr list…".into(),
            JobKind::Details(number) => format!("Running gh pr view {number}…"),
            JobKind::Checks(number) => format!("Running gh pr checks {number}…"),
            JobKind::Raw(command) => format!("Running {command}…"),
        };
        let tx = self.tx.clone();
        thread::spawn(move || {
            let _ = tx.send(execute_job(kind));
        });
    }

    pub fn receive(&mut self) -> bool {
        let mut changed = false;
        while let Ok(result) = self.rx.try_recv() {
            changed = true;
            self.loading = false;
            match result {
                JobResult::Overview {
                    git,
                    github,
                    items,
                    pr_message,
                } => {
                    self.git = git;
                    self.overview_github = github;
                    self.pull_requests = items;
                    self.pr_message = pr_message;
                    self.selected_pr = self
                        .selected_pr
                        .min(self.pull_requests.len().saturating_sub(1));
                    self.command_log = "Repository and GitHub status updated".into();
                }
                JobResult::Git(snapshot) => {
                    self.git = snapshot;
                    self.command_log = "Local repository updated".into();
                }
                JobResult::PullRequests { items, message } => {
                    self.pull_requests = items;
                    self.pr_message = message;
                    self.command_log = self
                        .pr_message
                        .lines()
                        .next()
                        .unwrap_or("gh pr list completed")
                        .to_string();
                    self.selected_pr = self
                        .selected_pr
                        .min(self.pull_requests.len().saturating_sub(1));
                    self.detail.clear();
                    self.detail_number = None;
                    self.checks.clear();
                    self.checks_number = None;
                }
                JobResult::Details { number, text } => {
                    if self.selected_pr().is_some_and(|pr| pr.number == number) {
                        self.detail = text;
                        self.detail_number = Some(number);
                        self.command_log = format!("gh pr view {number} completed");
                    }
                }
                JobResult::Checks { number, text } => {
                    if self.selected_pr().is_some_and(|pr| pr.number == number) {
                        self.checks = text;
                        self.checks_number = Some(number);
                        self.command_log = format!("gh pr checks {number} completed");
                    }
                }
                JobResult::Raw(text) => {
                    self.command_log = text.lines().take(2).collect::<Vec<_>>().join("  ·  ");
                    self.output = text;
                }
            }
            if let Some(kind) = self.pending.take() {
                self.run(kind);
            }
        }
        changed
    }

    pub fn handle_key(&mut self, code: KeyCode) -> bool {
        if let Some(input) = self.input.as_mut() {
            match code {
                KeyCode::Esc => self.input = None,
                KeyCode::Enter => {
                    let command = self.input.take().unwrap_or_default();
                    if !command.trim().is_empty() {
                        self.view = View::Output;
                        self.run(JobKind::Raw(command));
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(ch) => input.push(ch),
                _ => {}
            }
            return false;
        }

        match code {
            KeyCode::Char('q') => return true,
            KeyCode::Char(':') => self.input = Some(String::new()),
            KeyCode::Char('o' | '1') => self.open(View::Overview),
            KeyCode::Char('g' | '2') => self.open(View::Git),
            KeyCode::Char('3') => self.open(View::Branches),
            KeyCode::Char('4') => self.open(View::Commits),
            KeyCode::Char('5') => self.open(View::Stash),
            KeyCode::Char('p' | '6') => self.open(View::PullRequests),
            KeyCode::Char('c') => self.open(View::Checks),
            KeyCode::Char('7') => self.view = View::Output,
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::PageDown => self.move_selection(10),
            KeyCode::PageUp => self.move_selection(-10),
            KeyCode::Enter if self.view == View::PullRequests => {
                if let Some(number) = self.selected_pr().map(|pr| pr.number) {
                    self.run(JobKind::Details(number));
                }
            }
            KeyCode::Tab => {
                let next = match self.view {
                    View::Overview => View::Git,
                    View::Git => View::Branches,
                    View::Branches => View::Commits,
                    View::Commits => View::Stash,
                    View::Stash => View::PullRequests,
                    View::PullRequests => View::Checks,
                    View::Checks | View::Output => View::Overview,
                };
                self.open(next);
            }
            _ => {}
        }
        false
    }

    fn open(&mut self, view: View) {
        self.view = view;
        self.scroll = 0;
        self.refresh();
    }

    fn refresh(&mut self) {
        match self.view {
            View::Overview => self.run(JobKind::Overview),
            View::Git | View::Branches | View::Commits | View::Stash => self.run(JobKind::Git),
            View::PullRequests => self.run(JobKind::PullRequests),
            View::Checks => {
                if let Some(number) = self.selected_pr().map(|pr| pr.number) {
                    self.run(JobKind::Checks(number));
                } else {
                    self.notice = "Load the pull request list first (p)".into();
                }
            }
            View::Output => self.notice = "Use : to run another command".into(),
        }
    }

    fn move_selection(&mut self, delta: isize) {
        if matches!(self.view, View::PullRequests | View::Checks) && !self.pull_requests.is_empty()
        {
            let next = (self.selected_pr as isize + delta)
                .clamp(0, self.pull_requests.len() as isize - 1) as usize;
            if next != self.selected_pr {
                self.selected_pr = next;
                self.detail.clear();
                self.detail_number = None;
                self.checks.clear();
                self.checks_number = None;
                self.scroll = 0;
            }
        } else if delta < 0 {
            self.scroll = self.scroll.saturating_sub(delta.unsigned_abs() as u16);
        } else {
            self.scroll = self.scroll.saturating_add(delta as u16);
        }
    }
}
