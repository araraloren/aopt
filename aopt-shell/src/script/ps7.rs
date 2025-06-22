use super::Generator;

use crate::acore::Error;
use crate::SHELL_PSH7;

#[derive(Debug, Clone, Copy, Default)]
pub struct PowerShell7;

impl Generator for PowerShell7 {
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        name == SHELL_PSH7
    }

    fn generate(&self, name: &str, bin: &str) -> Result<String, Self::Err> {
        let template = r#"Register-ArgumentCompleter -Native -CommandName PROGRAM -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $words = $commandAst.CommandElements
    $curr = $wordToComplete
    $prev = 'PROGRAM'
    $cword = 0
    $index = 0
    $commandline = $commandAst.ToString()
    for (; $index -lt $cursorPosition; $index++) {
        if ($commandline[$index] -match '\s+') {
            $cword++
        }
    }
    if (-not [char]::IsWhiteSpace($commandline[$cursorPosition]) -and [string]::IsNullOrWhiteSpace($curr)) {
        $cword ++
    }
    if ($words.Count -gt 1) {
        $prev = if ([string]::IsNullOrWhiteSpace($curr)) { $words[-1] } else { $words[-2] }
    }
    try {
        $completions = & PROGRAM --_shell SHELL --_curr "$curr" --_prev "$prev" --_cword "$cword" $words

        if ($LASTEXITCODE -eq 0) {
            return $completions | ForEach-Object {
                $split = $_.Split("`t");
                $name = $split[0].Trim();
                $desc = if ($split.Length -gt 1) { $split[1] } else { $name }

                [System.Management.Automation.CompletionResult]::new($name, $name, 'ParameterValue', $desc)
            }
        }
    }
    catch { }
    
    return @()
};
"#;

        Ok(template
            .replace("NAME", name)
            .replace("PROGRAM", bin)
            .replace("SHELL", SHELL_PSH7))
    }
}
