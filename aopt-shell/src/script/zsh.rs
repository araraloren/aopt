use super::Generator;

use crate::acore::Error;

#[derive(Debug, Clone, Copy, Default)]
pub struct Zsh;

impl Generator for Zsh {
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        name == "powershell"
    }

    fn generate(&self, name: &str, bin: &str) -> Result<String, Self::Err> {
        let template = r#"#compdef NAME

function __complete_handler_NAME() {
    local curr prev index cword

    cword=${#words[@]}
    index=$CURRENT
    curr="${words[$index]}"
    prev="${words[$index - 1]}"

    local completions=("${(@f)$(PROGRAM --_shell zsh --_curr "$curr" --_prev "$prev" "${words[@]}")}")

    if [[ -n $completions ]]; then
        _describe 'values' completions
    else
        _files
    fi
}

if [[ $zsh_eval_context[-1] == loadautofunc ]]; then
  # autoload from fpath, call function directly, not sure how it works
    __complete_handler_NAME "$@"
else
  # eval/source/. command, register function for later
    compdef __complete_handler_NAME PROGRAM
fi
"#;

        Ok(template.replace("NAME", name).replace("PROGRAM", bin))
    }
}
