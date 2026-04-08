use std::path::PathBuf;

use regex::Regex;

use crate::config::Config;
use crate::mode::{MatchResult, ParserState};

pub struct GrepMode {
    line_re: Regex,
}

impl GrepMode {
    pub fn new() -> Self {
        Self {
            line_re: Regex::new(r"^(\d+):(\d+):").unwrap(),
        }
    }

    pub fn extra_args(&self, config: &Config) -> Vec<String> {
        match config.search_prog.as_str() {
            "ag" => vec!["--group".into(), "--color".into(), "--column".into()],
            "rg" => {
                let has_color = config.user_args.iter().any(|a| a.starts_with("--color"));
                let mut args = vec!["--heading".into(), "--column".into()];
                if !has_color {
                    args.push("--color".into());
                    args.push("always".into());
                }
                args
            }
            _ => vec![],
        }
    }

    pub fn parse_line(
        &self,
        _raw: &str,
        stripped: &str,
        state: &mut ParserState,
    ) -> Option<MatchResult> {
        if stripped.is_empty() {
            state.current_path = None;
            return None;
        }

        if state.current_path.is_none() {
            let path = PathBuf::from(stripped);
            let abs = std::fs::canonicalize(&path)
                .unwrap_or_else(|_| std::env::current_dir().unwrap().join(&path));
            state.current_path = Some(abs);
            return None;
        }

        if let Some(caps) = self.line_re.captures(stripped) {
            let line: u32 = caps[1].parse().ok()?;
            let column: u32 = caps[2].parse().ok()?;
            return Some(MatchResult {
                file: state.current_path.clone().unwrap(),
                line: Some(line),
                column: Some(column),
            });
        }

        None
    }

    pub fn default_cmd_fmt(&self) -> &str {
        r#"vim -c "call cursor({line}, {column})" "{file}""#
    }
}
