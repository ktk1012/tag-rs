mod alias;
mod ansi;
mod config;
mod mode;
mod modes;
mod runner;

use std::io::{self, BufRead, BufReader, Write};
use std::process;

use is_terminal::IsTerminal;
use owo_colors::OwoColorize;

use alias::AliasWriter;
use config::{Config, ModeKind};
use mode::{Mode, ParserState};
use modes::fd::FdMode;
use modes::grep::GrepMode;

fn run() -> i32 {
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tag-rs: {e}");
            return 1;
        }
    };

    let is_tty = io::stdin().is_terminal() && io::stdout().is_terminal();

    let mode = match config.mode {
        ModeKind::Grep => Mode::Grep(GrepMode::new()),
        ModeKind::Fd => Mode::Fd(FdMode::new()),
    };

    let extra = mode.extra_args(&config);
    let mut all_args = Vec::new();

    if is_tty && !config.disable_tag {
        all_args.extend(extra);
    }
    all_args.extend(config.user_args.iter().cloned());

    if config.disable_tag || !is_tty {
        let status = runner::passthrough(&config.search_prog, &all_args).unwrap_or_else(|e| {
            eprintln!("tag-rs: failed to run {}: {e}", config.search_prog);
            process::exit(1);
        });
        return runner::exit_code(status);
    }

    let mut child = runner::spawn_piped(&config.search_prog, &all_args).unwrap_or_else(|e| {
        eprintln!("tag-rs: failed to run {}: {e}", config.search_prog);
        process::exit(1);
    });

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut out = io::stdout().lock();

    let fmt_string = config
        .cmd_fmt_string
        .unwrap_or_else(|| mode.default_cmd_fmt().to_string());
    let mut alias_writer = AliasWriter::new(config.alias_prefix.clone(), fmt_string);
    let mut state = ParserState::new();
    let mut index: usize = 1;

    for line in reader.lines() {
        let raw = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let stripped_bytes = ansi::strip(raw.as_bytes());
        let stripped = String::from_utf8_lossy(&stripped_bytes);

        if let Some(result) = mode.parse_line(&raw, &stripped, &mut state) {
            alias_writer.write_alias(index, &result);
            let tag = format!("{}{}{} ", "[".blue(), index.red(), "]".blue(),);
            let _ = write!(out, "{tag}");
            let _ = writeln!(out, "{raw}");
            index += 1;
        } else {
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

fn main() {
    process::exit(run());
}
