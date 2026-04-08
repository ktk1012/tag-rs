# tag-rs

Rust rewrite of [tag](https://github.com/aykamko/tag) by [aykamko](https://github.com/aykamko).

`tag-rs` wraps search tools like [ripgrep](https://github.com/BurntSushi/ripgrep), [ag](https://github.com/ggreer/the_silver_searcher), and [fd](https://github.com/sharkdp/fd), tagging each match with a number and generating shell aliases so you can jump to any result instantly.

```
$ rg pattern src/
src/main.rs
[1] 12:5:    let pattern = &args[1];
[2] 28:9:    if pattern.is_empty() {

$ e1  # opens src/main.rs at line 12, column 5
```

```
$ fd -e rs src/
[1] src/alias.rs
[2] src/config.rs
[3] src/main.rs
[4] src/mode.rs

$ e3  # opens src/main.rs
```

### Git status with numbering

```
$ gs
# Staged
[1] A. src/new.rs

# Unstaged
[2] .M src/config.rs

# Untracked
[3] ?? notes.txt

$ git add $e2       # add src/config.rs by number
$ e1                # open src/new.rs in editor
```

### Git branch with numbering

```
$ gb
[1] * main
[2]   feature/xyz
[3]   fix/bug-42

$ git checkout $e3  # checkout fix/bug-42
```

### Number expansion

```
$ tag-rs expand 1 3-5
/absolute/path/to/a.rs /absolute/path/to/c.rs /absolute/path/to/d.rs /absolute/path/to/e.rs

$ git add $(tag-rs expand 1 3-5)  # add files 1, 3, 4, 5
```

## What's different from tag?

- Rewritten in Rust
- Supports `fd` in addition to `rg` and `ag` — tag file search results, not just grep matches
- Git integration: numbered `git status` (`gs`) and `git branch` (`gb`) with numeric file/branch references
- Number expansion: `tag-rs expand 1 3-5` resolves numbered references to paths for use with any command
- Simpler template syntax: `{file}`, `{line}`, `{column}` instead of `{{.Filename}}`, `{{.LineNumber}}`, `{{.ColumnNumber}}`

## Installation

### Nix flake

Add as a flake input and use the package output:

```nix
# flake.nix
{
  inputs.tag-rs.url = "github:ktk1012/tag-rs";

  # then in home-manager, nix-darwin, or NixOS config:
  # tag-rs.packages.${system}.default
}
```

Or run directly without installing:

```sh
nix run github:ktk1012/tag-rs -- pattern src/
```

### From source (with Nix)

```sh
git clone https://github.com/ktk1012/tag-rs.git
cd tag-rs
nix develop -c cargo install --path .
```

### From source (without Nix)

Requires Rust 1.85+.

```sh
git clone https://github.com/ktk1012/tag-rs.git
cd tag-rs
cargo install --path .
```

## Shell setup

`tag-rs` writes aliases to a file (default `/tmp/tag_aliases_<ppid>`, where `<ppid>` is the parent shell's PID). Your shell needs to source this file after each run. Add the snippet for your shell to your rc file (e.g. `~/.zshrc`).

- **zsh** — [`integrations/zsh.zsh`](integrations/zsh.zsh)
- **bash** — [`integrations/bash.bash`](integrations/bash.bash)
- **fish** — [`integrations/fish.fish`](integrations/fish.fish)

To use `ag` instead of `rg`, set `TAG_SEARCH_PROG=ag` and alias `ag` instead of `rg`.

## Configuration

| Variable | Default | Description |
|---|---|---|
| `TAG_SEARCH_PROG` | `rg` | Search backend: `rg`, `ag`, or `fd` |
| `TAG_ALIAS_FILE` | `/tmp/tag_aliases_<ppid>` | Path to generated alias file (default includes parent shell PID) |
| `TAG_ALIAS_PREFIX` | `e` | Prefix for aliases (e.g. `e1`, `e2`) |
| `TAG_CMD_FMT_STRING` | `vim -c "call cursor({line}, {column})" "{file}"` | Editor command template for grep mode (`rg`/`ag`) |
| `TAG_CMD_FMT_STRING_FD` | `vim "{file}"` | Editor command template for fd mode |

Template placeholders: `{file}`, `{line}`, `{column}`. Go template syntax from the original tag (`{{.Filename}}`, `{{.LineNumber}}`, `{{.ColumnNumber}}`) is also accepted.

## Acknowledgements

This project is inspired by and based on [tag](https://github.com/aykamko/tag) by [aykamko](https://github.com/aykamko), which is itself a reimagining of [sack](https://github.com/sampson-chen/sack). The original `tag` is licensed under the [MIT License](https://github.com/aykamko/tag/blob/master/LICENSE).

## License

[MIT](LICENSE)
