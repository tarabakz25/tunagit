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
| `Enter` | Show the selected pull request (`gh pr view`) |
| `c` | Show checks for the selected pull request (`gh pr checks`) |
| `:` | Run any `git ...` or `gh ...` command |
| `r` | Refresh the current view |
| `j` / `k`, arrows | Select a pull request or scroll output |
| `Tab` | Move to the next view |
| `q` | Quit |

The command prompt accepts quoted arguments, for example `gh pr checks 42` or
`git log --oneline -20`. Commands run in the current repository and show their
output and exit status in the TUI.

## Try the pull request view

From a repository with a GitHub remote, run `cargo run` and press `6` to load
open pull requests. Use `j` / `k` to select one, `Enter` to read its details,
and `c` to display its checks. Press `r` to refresh after checks change.
If a pull request has no checks, the checks pane shows the message from `gh`.

## Code structure

- `src/app.rs` owns view state and keyboard actions.
- `src/commands.rs` runs `git` and `gh` and prepares their results.
- `src/components/` contains the sidebar, detail, command log, footer, and
  theme components.
