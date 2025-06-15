use super::Generator;

use crate::acore::Error;

#[derive(Debug, Clone, Copy, Default)]
pub struct PowerShell;

impl Generator for PowerShell {
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        name == "powershell"
    }

    fn generate(&self, name: &str, bin: &str) -> Result<String, Self::Err> {
        let template = r#"
Register-ArgumentCompleter -CommandName PROGRAM -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $words = $commandAst.CommandElements;
    $currentWord = $wordToComplete;
    $prevWord = if ($words.Count -gt 1 -and $words[-2]) { $words[-2] } else { 'PROGRAM' };

    try {
        $completions = & PROGRAM --_shell powershell --_curr "`"$currentWord`"" --_prev "`"$prevWord`"" $words;

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

        Ok(template.replace("NAME", name).replace("PROGRAM", bin))
    }
}
