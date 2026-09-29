# tunagit

A terminal interface for working with a local Git repository and GitHub CLI in one place.
Its layout follows lazygit's vertical panel arrangement. Status, Files, Local
branches, Commits, Stash, and Pull requests stay on the left. The selected
panel's details fill the right side, with a small command log below.

## Run

```sh
cargo run
```

`git` and GitHub CLI (`gh`) must be installed and available in `PATH`. Sign in with
`gh auth login` to load pull requests and checks.
The interface uses terminal colors and honors `NO_COLOR`. If that variable is
set, remove it from the environment to see the colored panels.

## Keys

| Key | Action |
| --- | --- |
| `1` / `o` | Status: local status and `gh pr status` |
| `2` / `g` | Files and remotes |
| `3` | Local branches |
| `4` | Commits |
| `5` | Stash |
| `6` / `p` | Pull requests (`gh pr list`) |
| `Enter` | Show the selected pull request and focus its details (`gh pr view`) |
| `c` | Show checks for the selected pull request (`gh pr checks`) |
| `h` / `l`, left / right | Focus the PR list or the detail pane |
| `:` | Run any `git ...` or `gh ...` command |
| `r` | Refresh the current view |
| `j` / `k`, up / down | Select a PR in the list or scroll the focused detail pane |
| PageUp / PageDown | Move ten PRs or scroll the detail pane ten lines |
| Left click | Switch sidebar panels, select a PR, or focus a pane |
| Left click (same PR) | Show the selected pull request detail (like `Enter`) |
| Right click (PR) | Show checks for the PR (like `c`) |
| Mouse wheel | Scroll the list or detail pane under the pointer |
| `Tab` | Move to the next view |
| `q` | Quit |

The command prompt accepts quoted arguments, for example `gh pr checks 42` or
`git log --oneline -20`. Commands run in the current repository and show their
output and exit status in the TUI.

In the pull request view, press `l` (or click the right pane) to focus it and
scroll a long description with `j` / `k`, the arrow keys, PageUp / PageDown,
or the mouse wheel. Press `h` (or click the PR list) to return to the list.
Click a sidebar panel to switch views, click a PR to select it, and click the
selected PR again for its full detail. Right-click a PR for its checks.

## Code structure

- `src/app.rs` owns view state and keyboard actions.
- `src/commands.rs` runs `git` and `gh` and prepares their results.
- `src/components/` contains the sidebar, detail, command log, footer, and
  theme components.
