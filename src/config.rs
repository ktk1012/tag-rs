use std::env;
use std::path::PathBuf;

/// Convert Go template syntax to tag-rs syntax for backward compatibility.
fn normalize_fmt_string(s: String) -> String {
    s.replace("{{.Filename}}", "{file}")
        .replace("{{.LineNumber}}", "{line}")
        .replace("{{.ColumnNumber}}", "{column}")
        .replace("{{.MatchIndex}}", "{index}")
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModeKind {
    Grep,
    Fd,
}

pub struct Config {
    pub search_prog: String,
    pub mode: ModeKind,
    pub alias_file: PathBuf,
    pub alias_prefix: String,
    pub cmd_fmt_string: Option<String>,
    pub user_args: Vec<String>,
    pub disable_tag: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let search_prog =
            env::var("TAG_SEARCH_PROG").unwrap_or_else(|_| "rg".to_string());

        let mode = match search_prog.as_str() {
            "rg" | "ag" => ModeKind::Grep,
            "fd" => ModeKind::Fd,
            other => return Err(format!("unsupported TAG_SEARCH_PROG: {other}")),
        };

        let alias_file = env::var("TAG_ALIAS_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let ppid = std::os::unix::process::parent_id();
                PathBuf::from(format!("/tmp/tag_aliases_{ppid}"))
            });

        let alias_prefix =
            env::var("TAG_ALIAS_PREFIX").unwrap_or_else(|_| "e".to_string());

        let cmd_fmt_string = match mode {
            ModeKind::Fd => env::var("TAG_CMD_FMT_STRING_FD").ok(),
            ModeKind::Grep => env::var("TAG_CMD_FMT_STRING").ok(),
        }
        .map(normalize_fmt_string);

        let mut user_args: Vec<String> = env::args().skip(1).collect();
        let mut disable_tag = false;

        if let Some(pos) = user_args.iter().position(|a| a == "--notag") {
            user_args.remove(pos);
            disable_tag = true;
        }

        if user_args.is_empty() && mode == ModeKind::Grep {
            disable_tag = true;
        }

        Ok(Config {
            search_prog,
            mode,
            alias_file,
            alias_prefix,
            cmd_fmt_string,
            user_args,
            disable_tag,
        })
    }
}
