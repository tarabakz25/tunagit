use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use crossterm::event::{KeyCode, MouseButton, MouseEventKind};

use crate::commands::{GitSnapshot, JobKind, JobResult, PullRequest, execute_job};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PrFocus {
    List,
    Detail,
}

pub enum ScrollTarget {
    PrList,
    Detail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickTarget {
    SidebarView(View),
    PrIndex(usize),
    Detail,
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
    pub pr_focus: PrFocus,
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
            pr_focus: PrFocus::List,
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
            KeyCode::Char('h') | KeyCode::Left if self.is_pr_view() => {
                self.pr_focus = PrFocus::List;
            }
            KeyCode::Char('l') | KeyCode::Right if self.is_pr_view() => {
                self.pr_focus = PrFocus::Detail;
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::PageDown => self.move_selection(10),
            KeyCode::PageUp => self.move_selection(-10),
            KeyCode::Enter if self.view == View::PullRequests => {
                if let Some(number) = self.selected_pr().map(|pr| pr.number) {
                    self.pr_focus = PrFocus::Detail;
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
        self.pr_focus = if view == View::Checks {
            PrFocus::Detail
        } else {
            PrFocus::List
        };
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
        if self.is_pr_view() && self.pr_focus == PrFocus::List && !self.pull_requests.is_empty() {
            let next = (self.selected_pr as isize + delta)
                .clamp(0, self.pull_requests.len() as isize - 1) as usize;
            if next != self.selected_pr {
                self.select_pr(next);
            }
        } else {
            self.scroll_by(delta);
        }
    }

    fn scroll_by(&mut self, delta: isize) {
        if delta < 0 {
            self.scroll = self.scroll.saturating_sub(delta.unsigned_abs() as u16);
        } else {
            self.scroll = self.scroll.saturating_add(delta as u16);
        }
    }

    fn is_pr_view(&self) -> bool {
        matches!(self.view, View::PullRequests | View::Checks)
    }

    pub fn handle_mouse_scroll(&mut self, kind: MouseEventKind, target: ScrollTarget) -> bool {
        let delta = match kind {
            MouseEventKind::ScrollDown => 3,
            MouseEventKind::ScrollUp => -3,
            _ => return false,
        };
        match target {
            ScrollTarget::PrList if self.is_pr_view() => {
                self.pr_focus = PrFocus::List;
                self.move_selection(delta);
            }
            ScrollTarget::Detail => {
                if self.is_pr_view() {
                    self.pr_focus = PrFocus::Detail;
                }
                self.scroll_by(delta);
            }
            ScrollTarget::PrList => return false,
        }
        true
    }

    pub fn handle_mouse_click(&mut self, kind: MouseEventKind, target: ClickTarget) -> bool {
        if self.input.is_some() {
            return false;
        }
        let MouseEventKind::Down(button) = kind else {
            return false;
        };
        match button {
            MouseButton::Left => self.handle_left_click(target),
            MouseButton::Right | MouseButton::Middle => self.handle_right_click(target),
        }
    }

    fn handle_left_click(&mut self, target: ClickTarget) -> bool {
        match target {
            ClickTarget::SidebarView(view) => {
                if view == self.view {
                    if self.is_pr_view() && self.pr_focus != PrFocus::List {
                        self.pr_focus = PrFocus::List;
                        return true;
                    }
                    return false;
                }
                self.open(view);
                true
            }
            ClickTarget::PrIndex(index) => self.select_pr_with_mouse(index, false),
            ClickTarget::Detail => {
                if self.is_pr_view() && self.pr_focus != PrFocus::Detail {
                    self.pr_focus = PrFocus::Detail;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn handle_right_click(&mut self, target: ClickTarget) -> bool {
        match target {
            ClickTarget::PrIndex(index) => self.select_pr_with_mouse(index, true),
            ClickTarget::Detail if self.is_pr_view() => {
                if self.view == View::Checks {
                    return false;
                }
                self.open(View::Checks);
                true
            }
            ClickTarget::Detail | ClickTarget::SidebarView(_) => false,
        }
    }

    fn select_pr_with_mouse(&mut self, index: usize, open_checks: bool) -> bool {
        if index >= self.pull_requests.len() {
            return false;
        }
        if !self.is_pr_view() {
            self.view = View::PullRequests;
            self.pr_focus = PrFocus::List;
            self.scroll = 0;
            self.select_pr(index);
            if open_checks {
                self.open(View::Checks);
            } else {
                self.refresh();
            }
            return true;
        }
        if index != self.selected_pr {
            self.select_pr(index);
            self.pr_focus = PrFocus::List;
            if open_checks {
                self.open(View::Checks);
            }
            return true;
        }
        if open_checks {
            if self.view == View::Checks {
                return false;
            }
            self.open(View::Checks);
            return true;
        }
        self.pr_focus = PrFocus::Detail;
        let number = self.selected_pr().map(|pr| pr.number);
        match (self.view, number) {
            (View::PullRequests, Some(number)) => {
                self.run(JobKind::Details(number));
                true
            }
            (View::Checks, Some(number)) => {
                self.run(JobKind::Checks(number));
                true
            }
            _ => true,
        }
    }

    fn select_pr(&mut self, index: usize) {
        self.selected_pr = index;
        self.detail.clear();
        self.detail_number = None;
        self.checks.clear();
        self.checks_number = None;
        self.scroll = 0;
    }
}
