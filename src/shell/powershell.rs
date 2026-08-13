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
        }
        'deactivate' {
            $out = & $cvmBin __resolve-deactivate
            if ($LASTEXITCODE -ne 0) { return }
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
