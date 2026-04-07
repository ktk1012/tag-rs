# tag-rs

Rust rewrite of [tag](https://github.com/aykamko/tag) by [aykamko](https://github.com/aykamko).

`tag-rs` wraps search tools like [ripgrep](https://github.com/BurntSushi/ripgrep), [ag](https://github.com/ggreer/the_silver_searcher), and [fd](https://github.com/sharkdp/fd), tagging each match with a number and generating shell aliases so you can jump to any result instantly.

```
$ tag-rs pattern src/
src/main.rs
[1] 12:5:    let pattern = &args[1];
[2] 28:9:    if pattern.is_empty() {

$ e1  # opens src/main.rs at line 12, column 5
```

## What's different from tag?

- Rewritten in Rust
- Supports `fd` in addition to `rg` and `ag` — tag file search results, not just grep matches
- Simpler template syntax: `{file}`, `{line}`, `{column}` instead of `{{.Filename}}`, `{{.LineNumber}}`, `{{.ColumnNumber}}`

## Installation

### From source (with Nix)

```sh
git clone https://github.com/your-user/tag-rs.git
cd tag-rs
nix develop -c cargo install --path .
```

### From source (without Nix)

Requires Rust 1.85+.

```sh
git clone https://github.com/your-user/tag-rs.git
cd tag-rs
cargo install --path .
```

## Shell setup

`tag-rs` writes aliases to a file (default `/tmp/tag_aliases`). Your shell needs to source this file after each run.

### zsh

```zsh
if (( $+commands[tag-rs] )); then
  # grep mode (ripgrep)
  tag() { command tag-rs "$@"; source ${TAG_ALIAS_FILE:-/tmp/tag_aliases} 2>/dev/null }
  alias rg=tag

  # file find mode (fd)
  tagfd() { TAG_SEARCH_PROG=fd command tag-rs "$@"; source ${TAG_ALIAS_FILE:-/tmp/tag_aliases} 2>/dev/null }
  alias fd=tagfd
fi
```

### bash

```bash
if command -v tag-rs &>/dev/null; then
  # grep mode (ripgrep)
  tag() { command tag-rs "$@"; source ${TAG_ALIAS_FILE:-/tmp/tag_aliases} 2>/dev/null; }
  alias rg=tag

  # file find mode (fd)
  tagfd() { TAG_SEARCH_PROG=fd command tag-rs "$@"; source ${TAG_ALIAS_FILE:-/tmp/tag_aliases} 2>/dev/null; }
  alias fd=tagfd
fi
```

### fish

```fish
function tag
    set -q TAG_ALIAS_FILE; or set -l TAG_ALIAS_FILE /tmp/tag_aliases
    command tag-rs $argv; and source $TAG_ALIAS_FILE 2>/dev/null
end
alias rg tag

function tagfd
    set -q TAG_ALIAS_FILE; or set -l TAG_ALIAS_FILE /tmp/tag_aliases
    TAG_SEARCH_PROG=fd command tag-rs $argv; and source $TAG_ALIAS_FILE 2>/dev/null
end
alias fd tagfd
```

To use `ag` instead of `rg`, set `TAG_SEARCH_PROG=ag` and alias `ag` instead of `rg`.

## Configuration

| Variable | Default | Description |
|---|---|---|
| `TAG_SEARCH_PROG` | `rg` | Search backend: `rg`, `ag`, or `fd` |
| `TAG_ALIAS_FILE` | `/tmp/tag_aliases` | Path to generated alias file |
| `TAG_ALIAS_PREFIX` | `e` | Prefix for aliases (e.g. `e1`, `e2`) |
| `TAG_CMD_FMT_STRING` | _(mode default)_ | Template for editor command |

Default `TAG_CMD_FMT_STRING` per mode:

- **grep** (`rg`/`ag`): `vim -c "call cursor({line}, {column})" "{file}"`
- **fd**: `vim "{file}"`

## Acknowledgements

This project is inspired by and based on [tag](https://github.com/aykamko/tag) by [aykamko](https://github.com/aykamko), which is itself a reimagining of [sack](https://github.com/sampson-chen/sack). The original `tag` is licensed under the [MIT License](https://github.com/aykamko/tag/blob/master/LICENSE).

## License

[MIT](LICENSE)
