use super::Generator;

use crate::acore::Error;
use crate::SHELL_PSH;

#[derive(Debug, Clone, Copy, Default)]
pub struct PowerShell;

impl Generator for PowerShell {
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        name == SHELL_PSH
    }

    fn generate(&self, name: &str, bin: &str) -> Result<String, Self::Err> {
        let template = r#"Register-ArgumentCompleter -Native -CommandName PROGRAM -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $words = $commandAst.CommandElements;
    $curr = $wordToComplete;
    $prev = if ($words.Count -gt 1 -and $words[-2]) { $words[-2] } else { 'PROGRAM' };
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
    try {
        $completions = & PROGRAM --_shell SHELL --_curr "`"$curr`"" --_prev "`"$prev`"" --_cword "`"$cword`"" $words;

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
            .replace("SHELL", SHELL_PSH))
    }
}
