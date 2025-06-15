use super::Generator;

use crate::acore::Error;
use crate::SHELL_FISH;

#[derive(Debug, Clone, Copy, Default)]
pub struct Fish;

impl Generator for Fish {
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        name == SHELL_FISH
    }

    fn generate(&self, name: &str, bin: &str) -> Result<String, Self::Err> {
        let template = r#"function __complete_handler_NAME
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

    set -l completions (PROGRAM --_shell SHELL --_curr "$curr" --_prev "$prev" (string split " " -- $words))

    if test -n "$completions"
        string split '\n' -- $completions
    else
        __fish_complete_path "$curr" "paths"
    end
end

complete -f -c PROGRAM -a '(__complete_handler_NAME)'
"#;

        Ok(template
            .replace("NAME", name)
            .replace("PROGRAM", bin)
            .replace("SHELL", SHELL_FISH))
    }
}
