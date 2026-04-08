use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process;

use owo_colors::OwoColorize;

use crate::alias::AliasWriter;
use crate::config::SubcommandConfig;
use crate::runner;

#[derive(Debug, Clone, Copy, PartialEq)]
enum FileGroup {
    Staged,
    Unmerged,
    Unstaged,
    Untracked,
}

impl FileGroup {
    fn label(&self) -> &'static str {
        match self {
            FileGroup::Staged => "Staged",
            FileGroup::Unmerged => "Unmerged",
            FileGroup::Unstaged => "Unstaged",
            FileGroup::Untracked => "Untracked",
        }
    }

    fn filter_index(&self) -> u8 {
        match self {
            FileGroup::Staged => 1,
            FileGroup::Unmerged => 2,
            FileGroup::Unstaged => 3,
            FileGroup::Untracked => 4,
        }
    }
}

struct StatusEntry {
    group: FileGroup,
    code: String,
    path: PathBuf,
}

fn parse_porcelain_line(line: &str, cwd: &std::path::Path) -> Option<Vec<StatusEntry>> {
    if line.len() < 4 {
        return None;
    }

    let x = line.as_bytes()[0];
    let y = line.as_bytes()[1];
    let rest = &line[3..];

    // Handle renames: "R  old -> new" — use the new path
    let path_str = if rest.contains(" -> ") {
        rest.rsplit_once(" -> ").map(|(_, new)| new).unwrap_or(rest)
    } else {
        rest
    };

    let path = cwd.join(path_str);

    let mut entries = Vec::new();

    // Ignored files (shown with --ignored flag)
    if x == b'!' && y == b'!' {
        return None;
    }

    // Unmerged states (conflicts)
    if matches!((x, y), (b'U', _) | (_, b'U') | (b'A', b'A') | (b'D', b'D')) {
        entries.push(StatusEntry {
            group: FileGroup::Unmerged,
            code: format!("{}{}", x as char, y as char),
            path: path.clone(),
        });
        return Some(entries);
    }

    // Staged changes (index column)
    if x != b' ' && x != b'?' {
        entries.push(StatusEntry {
            group: FileGroup::Staged,
            code: format!("{}.", x as char),
            path: path.clone(),
        });
    }

    // Unstaged changes (worktree column)
    if y != b' ' && y != b'?' {
        entries.push(StatusEntry {
            group: FileGroup::Unstaged,
            code: format!(".{}", y as char),
            path: path.clone(),
        });
    }

    // Untracked
    if x == b'?' && y == b'?' {
        entries.push(StatusEntry {
            group: FileGroup::Untracked,
            code: "??".to_string(),
            path,
        });
    }

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

fn color_code(group: &FileGroup, code: &str) -> String {
    match group {
        FileGroup::Staged => code.green().to_string(),
        FileGroup::Unmerged => code.red().bold().to_string(),
        FileGroup::Unstaged => code.red().to_string(),
        FileGroup::Untracked => code.bright_black().to_string(),
    }
}

fn color_path(group: &FileGroup, path: &str) -> String {
    match group {
        FileGroup::Staged => path.green().to_string(),
        FileGroup::Unmerged => path.red().bold().to_string(),
        FileGroup::Unstaged => path.red().to_string(),
        FileGroup::Untracked => path.bright_black().to_string(),
    }
}

pub fn run(config: SubcommandConfig) -> i32 {
    let filter: Option<u8> = config
        .args
        .first()
        .and_then(|a| a.parse().ok())
        .filter(|&n: &u8| (1..=4).contains(&n));

    // Pass extra args to git status, skipping the first arg if it was consumed as a filter
    let mut git_args = vec!["status".to_string(), "--porcelain".to_string()];
    let skip_first = filter.is_some();
    for (i, arg) in config.args.iter().enumerate() {
        if i == 0 && skip_first {
            continue;
        }
        git_args.push(arg.clone());
    }

    let mut child = runner::spawn_piped("git", &git_args).unwrap_or_else(|e| {
        eprintln!("tag-rs: failed to run git status: {e}");
        process::exit(1);
    });

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    let cwd = std::env::current_dir().unwrap_or_default();

    // Collect all entries first for grouped display
    let mut all_entries: Vec<StatusEntry> = Vec::new();

    for line in reader.lines() {
        let raw = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if let Some(entries) = parse_porcelain_line(&raw, &cwd) {
            all_entries.extend(entries);
        }
    }

    let status = child.wait().unwrap_or_else(|e| {
        eprintln!("tag-rs: wait failed: {e}");
        process::exit(1);
    });

    if all_entries.is_empty() {
        println!("nothing to commit, working tree clean");
        return runner::exit_code(status);
    }

    let mut out = io::stdout().lock();
    let fmt_string = r#"vim "{file}""#.to_string();
    let mut alias_writer = AliasWriter::new(config.alias_prefix.clone(), fmt_string);
    let mut index: usize = 1;

    let groups = [
        FileGroup::Staged,
        FileGroup::Unmerged,
        FileGroup::Unstaged,
        FileGroup::Untracked,
    ];

    for group in &groups {
        if let Some(f) = filter
            && group.filter_index() != f
        {
            continue;
        }

        let entries: Vec<&StatusEntry> = all_entries.iter().filter(|e| e.group == *group).collect();

        if entries.is_empty() {
            continue;
        }

        let header = format!("# {}", group.label());
        let _ = writeln!(out, "{}", header.bold());

        for entry in entries {
            let display_path = entry
                .path
                .strip_prefix(&cwd)
                .unwrap_or(&entry.path)
                .display()
                .to_string();

            let tag = format!("{}{}{}", "[".blue(), index.red(), "]".blue());
            let code = color_code(group, &entry.code);
            let path = color_path(group, &display_path);
            let _ = writeln!(out, "{tag} {code} {path}");

            let result = crate::mode::MatchResult {
                file: entry.path.clone(),
                line: None,
                column: None,
            };
            alias_writer.write_alias(index, &result);
            index += 1;
        }

        let _ = writeln!(out);
    }

    let _ = alias_writer.flush_to_file(&config.alias_file);

    runner::exit_code(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn cwd() -> PathBuf {
        PathBuf::from("/test")
    }

    #[test]
    fn parse_modified_unstaged() {
        let entries = parse_porcelain_line(" M src/main.rs", &cwd()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].group, FileGroup::Unstaged);
        assert_eq!(entries[0].code, ".M");
    }

    #[test]
    fn parse_modified_staged() {
        let entries = parse_porcelain_line("M  src/main.rs", &cwd()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].group, FileGroup::Staged);
        assert_eq!(entries[0].code, "M.");
    }

    #[test]
    fn parse_both_staged_and_unstaged() {
        let entries = parse_porcelain_line("MM src/main.rs", &cwd()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].group, FileGroup::Staged);
        assert_eq!(entries[1].group, FileGroup::Unstaged);
    }

    #[test]
    fn parse_added() {
        let entries = parse_porcelain_line("A  src/new.rs", &cwd()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].group, FileGroup::Staged);
        assert_eq!(entries[0].code, "A.");
    }

    #[test]
    fn parse_untracked() {
        let entries = parse_porcelain_line("?? unknown.txt", &cwd()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].group, FileGroup::Untracked);
        assert_eq!(entries[0].code, "??");
    }

    #[test]
    fn parse_unmerged_both_added() {
        let entries = parse_porcelain_line("AA conflict.rs", &cwd()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].group, FileGroup::Unmerged);
    }

    #[test]
    fn parse_unmerged_both_modified() {
        let entries = parse_porcelain_line("UU conflict.rs", &cwd()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].group, FileGroup::Unmerged);
    }

    #[test]
    fn parse_rename() {
        let entries = parse_porcelain_line("R  old.rs -> new.rs", &cwd()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].group, FileGroup::Staged);
        assert!(entries[0].path.ends_with("new.rs"));
    }

    #[test]
    fn parse_deleted_staged() {
        let entries = parse_porcelain_line("D  removed.rs", &cwd()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].group, FileGroup::Staged);
        assert_eq!(entries[0].code, "D.");
    }

    #[test]
    fn parse_short_line_returns_none() {
        assert!(parse_porcelain_line("", &cwd()).is_none());
        assert!(parse_porcelain_line("AB", &cwd()).is_none());
    }

    #[test]
    fn parse_ignored_returns_none() {
        assert!(parse_porcelain_line("!! build/", &cwd()).is_none());
    }
}
