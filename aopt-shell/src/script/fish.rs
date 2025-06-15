use super::Generator;

use crate::acore::Error;

#[derive(Debug, Clone, Copy, Default)]
pub struct Fish;

impl Generator for Fish {
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        name == "bash"
    }

    fn generate(&self, name: &str, bin: &str) -> Result<String, Self::Err> {
        let template = r#"#!/usr/bin/env fish

function __complete_handler_NAME
    set -l words
    set -l curr
    set -l tokens
    set -l tokenCount

    if commandline -x >/dev/null 2>&1
        set curr (commandline -xpt)
        set words (commandline -xp)

        set tokens (commandline -xc)
    else
        set curr (commandline -opt)
        set words (commandline -op)

        set tokens (commandline -oc)
    end

    set -l cword (count $words)
    set -l prev $tokens[-1]

    PROGRAM --_shell fish --_curr "$curr" --_prev "$prev" (string split " " -- $words)
end

complete -f -c fput -a '(__complete_handler_NAME)'
        "#;

        Ok(template.replace("NAME", name).replace("PROGRAM", bin))
    }
}
