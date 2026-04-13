use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

use crate::config::ExpandConfig;

pub fn run(config: ExpandConfig) -> i32 {
    if config.args.is_empty() {
        eprintln!("tag-rs expand: no arguments given");
        return 1;
    }

    let vars = match load_vars(&config.alias_file, &config.alias_prefix) {
        Ok(v) => v,
        Err(e) => {
            if needs_expansion(&config.args) {
                eprintln!("tag-rs expand: {e}");
                return 1;
            }
            HashMap::new()
        }
    };

    let expanded = match expand_args(&config.args, &vars) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("tag-rs expand: {e}");
            return 1;
        }
    };

    let mut out = io::stdout().lock();
    let _ = writeln!(out, "{}", expanded.join(" "));
    0
}

/// Load exported variables from the alias file.
/// Parses lines like: `export e1='/path/to/file'`
fn load_vars(path: &std::path::Path, prefix: &str) -> Result<HashMap<usize, String>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;

    let mut vars = HashMap::new();
    let export_prefix = format!("export {prefix}");

    for line in content.lines() {
        let Some(rest) = line.strip_prefix(&export_prefix) else {
            continue;
        };
        // rest looks like: "3='/some/path'"
        let Some((num_str, value)) = rest.split_once('=') else {
            continue;
        };
        let Ok(num) = num_str.parse::<usize>() else {
            continue;
        };
        // Strip one pair of surrounding quotes
        let value = value
            .strip_prefix('\'')
            .and_then(|v| v.strip_suffix('\''))
            .or_else(|| value.strip_prefix('"').and_then(|v| v.strip_suffix('"')))
            .unwrap_or(value);
        // Unescape shell-escaped single quotes ('\'' → ')
        let value = value.replace("'\\''", "'");
        vars.insert(num, value);
    }

    Ok(vars)
}

/// Expand arguments: numbers become paths, ranges expand, non-numbers pass through.
fn expand_args(args: &[String], vars: &HashMap<usize, String>) -> Result<Vec<String>, String> {
    let mut result = Vec::new();

    for arg in args {
        if let Some((start, end)) = parse_range(arg) {
            for i in start..=end {
                match vars.get(&i) {
                    Some(v) => result.push(v.clone()),
                    None => return Err(format!("no file for index {i}")),
                }
            }
        } else if let Ok(num) = arg.parse::<usize>() {
            match vars.get(&num) {
                Some(v) => result.push(v.clone()),
                None => return Err(format!("no file for index {num}")),
            }
        } else {
            // Non-numeric argument — pass through as-is
            result.push(arg.clone());
        }
    }

    Ok(result)
}

/// Check if any argument requires numeric expansion (is a number or range).
fn needs_expansion(args: &[String]) -> bool {
    args.iter()
        .any(|a| a.parse::<usize>().is_ok() || parse_range(a).is_some())
}

/// Parse a range like "3-5" into (3, 5). Returns None if not a range.
fn parse_range(s: &str) -> Option<(usize, usize)> {
    let (a, b) = s.split_once('-')?;
    let start: usize = a.parse().ok()?;
    let end: usize = b.parse().ok()?;
    if start <= end {
        Some((start, end))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vars() -> HashMap<usize, String> {
        let mut m = HashMap::new();
        m.insert(1, "/path/to/a.rs".to_string());
        m.insert(2, "/path/to/b.rs".to_string());
        m.insert(3, "/path/to/c.rs".to_string());
        m.insert(4, "/path/to/d.rs".to_string());
        m.insert(5, "/path/to/e.rs".to_string());
        m
    }

    #[test]
    fn expand_single_number() {
        let vars = make_vars();
        let args = vec!["1".to_string()];
        let result = expand_args(&args, &vars).unwrap();
        assert_eq!(result, vec!["/path/to/a.rs"]);
    }

    #[test]
    fn expand_range() {
        let vars = make_vars();
        let args = vec!["2-4".to_string()];
        let result = expand_args(&args, &vars).unwrap();
        assert_eq!(
            result,
            vec!["/path/to/b.rs", "/path/to/c.rs", "/path/to/d.rs"]
        );
    }

    #[test]
    fn expand_mixed() {
        let vars = make_vars();
        let args = vec![
            "1".to_string(),
            "3-4".to_string(),
            "somefile.txt".to_string(),
        ];
        let result = expand_args(&args, &vars).unwrap();
        assert_eq!(
            result,
            vec![
                "/path/to/a.rs",
                "/path/to/c.rs",
                "/path/to/d.rs",
                "somefile.txt"
            ]
        );
    }

    #[test]
    fn expand_passthrough_non_numeric() {
        let vars = make_vars();
        let args = vec!["--flag".to_string(), "file.txt".to_string()];
        let result = expand_args(&args, &vars).unwrap();
        assert_eq!(result, vec!["--flag", "file.txt"]);
    }

    #[test]
    fn expand_missing_index_errors() {
        let vars = make_vars();
        let args = vec!["99".to_string()];
        assert!(expand_args(&args, &vars).is_err());
    }

    #[test]
    fn parse_range_valid() {
        assert_eq!(parse_range("3-5"), Some((3, 5)));
    }

    #[test]
    fn parse_range_single() {
        assert_eq!(parse_range("3-3"), Some((3, 3)));
    }

    #[test]
    fn parse_range_invalid_reversed() {
        assert_eq!(parse_range("5-3"), None);
    }

    #[test]
    fn parse_range_not_a_range() {
        assert_eq!(parse_range("abc"), None);
        assert_eq!(parse_range("3"), None);
    }

    #[test]
    fn load_vars_parses_export_lines() {
        let dir = std::env::temp_dir();
        let path = dir.join("tag_test_vars");
        fs::write(
            &path,
            "unset e1 e2 e3\nalias e1='vim /a.rs'\nexport e1='/a.rs'\nalias e2='vim /b.rs'\nexport e2='/b.rs'\n",
        )
        .unwrap();

        let vars = load_vars(&path, "e").unwrap();
        assert_eq!(vars.get(&1).unwrap(), "/a.rs");
        assert_eq!(vars.get(&2).unwrap(), "/b.rs");
        assert_eq!(vars.len(), 2);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn needs_expansion_with_numbers() {
        let args = vec!["3".to_string()];
        assert!(needs_expansion(&args));
    }

    #[test]
    fn needs_expansion_with_range() {
        let args = vec!["2-4".to_string()];
        assert!(needs_expansion(&args));
    }

    #[test]
    fn needs_expansion_without_numbers() {
        let args = vec!["main".to_string(), "--flag".to_string()];
        assert!(!needs_expansion(&args));
    }

    #[test]
    fn needs_expansion_mixed() {
        let args = vec!["main".to_string(), "3".to_string()];
        assert!(needs_expansion(&args));
    }

    #[test]
    fn expand_passthrough_without_alias_file() {
        // When no vars loaded, non-numeric args should pass through
        let vars = HashMap::new();
        let args = vec!["main".to_string(), "-c".to_string()];
        let result = expand_args(&args, &vars).unwrap();
        assert_eq!(result, vec!["main", "-c"]);
    }
}
