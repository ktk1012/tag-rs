use std::io::{self, BufRead, BufReader, Write};
use std::process;

use owo_colors::OwoColorize;

use crate::alias::AliasWriter;
use crate::config::SubcommandConfig;
use crate::runner;

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

        let (is_current, branch_name) = parse_branch_line(&raw);

        if let Some(name) = branch_name {
            alias_writer.write_var(index, &name);
            let tag = format!("{}{}{}", "[".blue(), index.red(), "]".blue());
            if is_current {
                let _ = writeln!(out, "{tag} {} {}", "*".green(), name.green());
            } else {
                let _ = writeln!(out, "{tag}   {name}");
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

/// Parse a `git branch` output line. Returns (is_current, branch_name).
fn parse_branch_line(line: &str) -> (bool, Option<String>) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return (false, None);
    }

    if let Some(rest) = trimmed.strip_prefix("* ") {
        // Detached HEAD: "* (HEAD detached at ...)"
        if rest.starts_with('(') {
            return (true, None);
        }
        (true, Some(rest.to_string()))
    } else {
        (false, Some(trimmed.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_regular_branch() {
        let (current, name) = parse_branch_line("  main");
        assert!(!current);
        assert_eq!(name.unwrap(), "main");
    }

    #[test]
    fn parse_current_branch() {
        let (current, name) = parse_branch_line("* feature/xyz");
        assert!(current);
        assert_eq!(name.unwrap(), "feature/xyz");
    }

    #[test]
    fn parse_detached_head() {
        let (current, name) = parse_branch_line("* (HEAD detached at abc1234)");
        assert!(current);
        assert!(name.is_none());
    }

    #[test]
    fn parse_empty_line() {
        let (current, name) = parse_branch_line("");
        assert!(!current);
        assert!(name.is_none());
    }
}
