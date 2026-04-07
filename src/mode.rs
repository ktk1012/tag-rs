use std::path::PathBuf;

use crate::config::Config;
use crate::modes::fd::FdMode;
use crate::modes::grep::GrepMode;

pub struct MatchResult {
    pub file: PathBuf,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

pub struct ParserState {
    pub current_path: Option<PathBuf>,
}

impl ParserState {
    pub fn new() -> Self {
        Self { current_path: None }
    }
}

pub enum Mode {
    Grep(GrepMode),
    Fd(FdMode),
}

impl Mode {
    pub fn extra_args(&self, config: &Config) -> Vec<String> {
        match self {
            Mode::Grep(m) => m.extra_args(config),
            Mode::Fd(m) => m.extra_args(config),
        }
    }

    pub fn parse_line(
        &self,
        raw: &str,
        stripped: &str,
        state: &mut ParserState,
    ) -> Option<MatchResult> {
        match self {
            Mode::Grep(m) => m.parse_line(raw, stripped, state),
            Mode::Fd(m) => m.parse_line(raw, stripped, state),
        }
    }

    pub fn default_cmd_fmt(&self) -> &str {
        match self {
            Mode::Grep(m) => m.default_cmd_fmt(),
            Mode::Fd(m) => m.default_cmd_fmt(),
        }
    }
}
