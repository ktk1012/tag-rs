use std::path::PathBuf;

use crate::config::Config;
use crate::mode::{MatchResult, ParserState};

pub struct FdMode;

impl FdMode {
    pub fn new() -> Self {
        Self
    }

    pub fn extra_args(&self, config: &Config) -> Vec<String> {
        let has_color = config
            .user_args
            .iter()
            .any(|a| a.starts_with("--color"));
        if has_color {
            vec![]
        } else {
            vec!["--color".into(), "always".into()]
        }
    }

    pub fn parse_line(
        &self,
        _raw: &str,
        stripped: &str,
        _state: &mut ParserState,
    ) -> Option<MatchResult> {
        if stripped.is_empty() {
            return None;
        }

        let path = PathBuf::from(stripped);
        let abs = std::fs::canonicalize(&path)
            .unwrap_or_else(|_| std::env::current_dir().unwrap().join(&path));

        Some(MatchResult {
            file: abs,
            line: None,
            column: None,
        })
    }

    pub fn default_cmd_fmt(&self) -> &str {
        r#"vim "{file}""#
    }
}
