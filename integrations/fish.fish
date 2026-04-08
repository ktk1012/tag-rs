function __tag_source
    set -q TAG_ALIAS_FILE; or set -l TAG_ALIAS_FILE /tmp/tag_aliases_$fish_pid
    source $TAG_ALIAS_FILE 2>/dev/null
end

function tag
    command tag-rs $argv; and __tag_source
end
alias rg tag

function tagfd
    TAG_SEARCH_PROG=fd command tag-rs $argv; and __tag_source
end
alias fd tagfd

# git status / branch with numbering
function gs
    command tag-rs gs $argv; and __tag_source
end

function gb
    command tag-rs gb $argv; and __tag_source
end

# git shortcuts with numeric file expansion (e.g. ga 1 3-5)
function ga
    git add (command tag-rs expand $argv)
end

function gd
    git diff (command tag-rs expand $argv)
end

function gds
    git diff --staged (command tag-rs expand $argv)
end

function grs
    git reset (command tag-rs expand $argv)
end

function gco
    git checkout (command tag-rs expand $argv)
end

function grm
    git rm (command tag-rs expand $argv)
end

function gbl
    git blame (command tag-rs expand $argv)
end

function gsw
    git switch (command tag-rs expand $argv)
end

function __tag_cleanup --on-event fish_exit
    rm -f {$TAG_ALIAS_FILE,/tmp/tag_aliases_$fish_pid}
end
