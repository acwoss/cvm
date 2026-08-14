pub const SCRIPT: &str = r#"# cvm shell integration for PowerShell
# Add this to your $PROFILE:
#   cvm init powershell | Out-String | Invoke-Expression

function cvm {
    param(
        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$CvmArgs
    )

    $cvmBin = (Get-Command cvm -CommandType Application -ErrorAction SilentlyContinue |
        Select-Object -First 1).Source
    if (-not $cvmBin) {
        Write-Error "cvm binary not found in PATH"
        return
    }

    switch ($CvmArgs[0]) {
        { $_ -in @('use', 'activate') } {
            if (-not $CvmArgs[1]) {
                Write-Error "Usage: cvm use <env_name>"
                return
            }
            $out = & $cvmBin __resolve-activate $CvmArgs[1]
            if ($LASTEXITCODE -ne 0) { return }
            foreach ($line in $out) {
                if ($line -match '^([^=]+)=(.*)$') {
                    Set-Item -Path "Env:$($Matches[1])" -Value $Matches[2]
                }
            }
            if (-not (Test-Path Env:CVM_OLD_PATH)) {
                $env:CVM_OLD_PATH = $env:Path
            }
            $env:Path = (Join-Path $env:CLAUDE_CONFIG_DIR 'bin') +
                [IO.Path]::PathSeparator + $env:CVM_OLD_PATH
            if (-not (Test-Path variable:global:CVM_OLD_PROMPT)) {
                $global:CVM_OLD_PROMPT = (Get-Content function:prompt).ToString()
            }
            function global:prompt {
                "($env:CVM_ENV) " + (& ([scriptblock]::Create($global:CVM_OLD_PROMPT)))
            }
        }
        'deactivate' {
            $out = & $cvmBin __resolve-deactivate
            if ($LASTEXITCODE -ne 0) { return }
            if (Test-Path Env:CVM_OLD_PATH) {
                $env:Path = $env:CVM_OLD_PATH
                Remove-Item Env:CVM_OLD_PATH
            }
            if (Test-Path variable:global:CVM_OLD_PROMPT) {
                Set-Item function:global:prompt ([scriptblock]::Create($global:CVM_OLD_PROMPT))
                Remove-Variable CVM_OLD_PROMPT -Scope Global
            }
            foreach ($line in $out) {
                if ($line) { Remove-Item -Path "Env:$line" -ErrorAction SilentlyContinue }
            }
        }
        default {
            & $cvmBin @CvmArgs
        }
    }
}
"#;
