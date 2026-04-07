# tag-rs v0.1.0

CLI utility that tags ripgrep/fd search results with numbers and generates shell aliases to open files instantly.

## Hard Rules (never bend)
- Never alter the original output format of ripgrep/fd
- Numbering must always start at 1
- When piped (non-TTY), pass through raw output without numbering

## Quick Ref
- Entry: `src/main.rs`
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy`
- Format: `cargo fmt`
- Nix build: `nix build`
- Nix dev shell: `nix develop` (direnv configured, auto-activates)

## Dev Conventions
- Tests before merge. Never declare done without a passing test.
- Commit only when explicitly requested.
- Logs: append-only (never overwrite log files).

## Compact Instructions
Preserve on compaction:
1. Hard Rules
2. Current active branch / uncommitted file list
3. Pending tasks and their status
4. Active errors or bugs being investigated
5. Dev Conventions
6. File paths modified in this session
