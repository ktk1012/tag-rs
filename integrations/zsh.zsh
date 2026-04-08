if (( $+commands[tag-rs] )); then
  _tag_source() { source ${TAG_ALIAS_FILE:-/tmp/tag_aliases_$$} 2>/dev/null }

  # grep mode (ripgrep)
  tag() { command tag-rs "$@"; _tag_source }
  alias rg=tag

  # file find mode (fd)
  tagfd() { TAG_SEARCH_PROG=fd command tag-rs "$@"; _tag_source }
  alias fd=tagfd

  # git status / branch with numbering
  gs() { command tag-rs gs "$@"; _tag_source }
  gb() { command tag-rs gb "$@"; _tag_source }

  # git shortcuts with numeric file expansion (e.g. ga 1 3-5)
  ga()  { git add $(command tag-rs expand "$@") }
  gd()  { git diff $(command tag-rs expand "$@") }
  gds() { git diff --staged $(command tag-rs expand "$@") }
  grs() { git reset $(command tag-rs expand "$@") }
  gco() { git checkout $(command tag-rs expand "$@") }
  grm() { git rm $(command tag-rs expand "$@") }
  gbl() { git blame $(command tag-rs expand "$@") }
  gsw() { git switch $(command tag-rs expand "$@") }

  trap 'rm -f "${TAG_ALIAS_FILE:-/tmp/tag_aliases_$$}"' EXIT
fi
