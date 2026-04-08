use std::io::{self, BufRead, BufReader, Write};
use std::process;

use owo_colors::OwoColorize;

use crate::alias::AliasWriter;
use crate::config::SubcommandConfig;
use crate::runner;

#[derive(Debug, Clone, Copy, PartialEq)]
enum BranchKind {
    /// Current branch (`*`)
    Current,
    /// Checked out in another worktree (`+`)
    Worktree,
    /// Regular branch
    Regular,
}

pub fn run(config: SubcommandConfig) -> i32 {
    let mut git_args = vec!["branch".to_string()];
    git_args.extend(config.args.iter().cloned());

    let mut child = runner::spawn_piped("git", &git_args).unwrap_or_else(|e| {
        eprintln!("tag-rs: failed to run git branch: {e}");
        process::exit(1);
    });

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut out = io::stdout().lock();

    // For branches, the alias just exports the branch name (no file to open).
    let fmt_string = "{file}".to_string();
    let mut alias_writer = AliasWriter::new(config.alias_prefix.clone(), fmt_string);
    let mut index: usize = 1;

    for line in reader.lines() {
        let raw = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let (kind, branch_name) = parse_branch_line(&raw);

        if let Some(name) = branch_name {
            alias_writer.write_var(index, &name);
            let tag = format!("{}{}{}", "[".blue(), index.red(), "]".blue());
            match kind {
                BranchKind::Current => {
                    let _ = writeln!(out, "{tag} {} {}", "*".green(), name.green());
                }
                BranchKind::Worktree => {
                    let _ = writeln!(out, "{tag} {} {name}", "+".cyan());
                }
                BranchKind::Regular => {
                    let _ = writeln!(out, "{tag}   {name}");
                }
            }
            index += 1;
        } else {
            // Lines we can't parse (e.g. detached HEAD notice) — pass through
            let _ = writeln!(out, "{raw}");
        }
    }

    let _ = alias_writer.flush_to_file(&config.alias_file);

    let status = child.wait().unwrap_or_else(|e| {
        eprintln!("tag-rs: wait failed: {e}");
        process::exit(1);
    });

    runner::exit_code(status)
}

/// Parse a `git branch` output line.
///
/// `git branch` output format: each line starts with a 2-char indicator
/// followed by a space and the branch name.
///   `* ` — current branch
///   `+ ` — checked out in another worktree
///   `  ` — regular branch
fn parse_branch_line(line: &str) -> (BranchKind, Option<String>) {
    if line.len() < 3 {
        return (BranchKind::Regular, None);
    }

    let indicator = line.as_bytes()[0];
    let name = line[2..].to_string();

    if name.is_empty() {
        return (BranchKind::Regular, None);
    }

    match indicator {
        b'*' => {
            // Detached HEAD: "* (HEAD detached at ...)"
            if name.starts_with('(') {
                (BranchKind::Current, None)
            } else {
                (BranchKind::Current, Some(name))
            }
        }
        b'+' => (BranchKind::Worktree, Some(name)),
        _ => (BranchKind::Regular, Some(name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_regular_branch() {
        let (kind, name) = parse_branch_line("  main");
        assert_eq!(kind, BranchKind::Regular);
        assert_eq!(name.unwrap(), "main");
    }

    #[test]
    fn parse_current_branch() {
        let (kind, name) = parse_branch_line("* feature/xyz");
        assert_eq!(kind, BranchKind::Current);
        assert_eq!(name.unwrap(), "feature/xyz");
    }

    #[test]
    fn parse_worktree_branch() {
        let (kind, name) = parse_branch_line("+ LAM-35/deploy-refactor");
        assert_eq!(kind, BranchKind::Worktree);
        assert_eq!(name.unwrap(), "LAM-35/deploy-refactor");
    }

    #[test]
    fn parse_detached_head() {
        let (kind, name) = parse_branch_line("* (HEAD detached at abc1234)");
        assert_eq!(kind, BranchKind::Current);
        assert!(name.is_none());
    }

    #[test]
    fn parse_empty_line() {
        let (kind, name) = parse_branch_line("");
        assert_eq!(kind, BranchKind::Regular);
        assert!(name.is_none());
    }

    #[test]
    fn parse_short_line() {
        let (kind, name) = parse_branch_line("* ");
        assert_eq!(kind, BranchKind::Regular);
        assert!(name.is_none());
    }
}
