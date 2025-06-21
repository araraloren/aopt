use super::Generator;

use crate::acore::Error;
use crate::SHELL_BASH;

#[derive(Debug, Clone, Copy, Default)]
pub struct Bash;

impl Generator for Bash {
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        name == SHELL_BASH
    }

    fn generate(&self, name: &str, bin: &str) -> Result<String, Self::Err> {
        let template = r#"#!/usr/bin/env bash

__completion_handle_NAME()
{
    local cword words cur prev

    major=${BASH_VERSINFO[0]}
    minor=${BASH_VERSINFO[1]}

    if [[ $major -gt 2 ]] || [[ $major -eq 2 && $minor -ge 12 ]]; then
        _comp_get_words -n '=' cur prev words cword
    else
        _get_comp_words_by_ref -n '=' cur prev words cword
    fi

    COMPREPLY=( $( PROGRAM --_shell SHELL --_curr "$cur" --_prev "$prev" --_cword "$cword" "${words[@]}" ) )
    if [[ $? != 0 ]]; then
        unset COMPREPLY
    fi
}

complete -o nospace -o default -F __completion_handle_NAME PROGRAM
"#;

        Ok(template
            .replace("NAME", name)
            .replace("PROGRAM", bin)
            .replace("SHELL", SHELL_BASH))
    }
}
