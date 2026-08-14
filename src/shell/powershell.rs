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
            if (-not (Test-Path Env:CVM_OLD_PATH)) {
                $env:CVM_OLD_PATH = $env:Path
            }
            foreach ($line in $out) {
                if ($line -match '^([^=]+)=(.*)$') {
                    $key = $Matches[1]
                    $reserved = @(
                        'PATH', 'PS1', 'CVM_OLD_PATH', 'CVM_OLD_PS1',
                        'CVM_OLD_PROMPT', 'CVM_HOME', 'CVM_AUTO',
                        'CVM_AUTO_ROOT', 'CVM_AUTO_LAST_PWD'
                    )
                    if ($key -notin $reserved) {
                        Set-Item -Path "Env:$key" -Value $Matches[2]
                    }
                }
            }
            $env:Path = (Join-Path $env:CLAUDE_CONFIG_DIR 'bin') +
                [IO.Path]::PathSeparator + $env:CVM_OLD_PATH
            Remove-Item Env:CVM_AUTO -ErrorAction SilentlyContinue
            Remove-Item Env:CVM_AUTO_ROOT -ErrorAction SilentlyContinue
        }
        'deactivate' {
            $out = & $cvmBin __resolve-deactivate
            if ($LASTEXITCODE -ne 0) { return }
            if (Test-Path Env:CVM_OLD_PATH) {
                $env:Path = $env:CVM_OLD_PATH
                Remove-Item Env:CVM_OLD_PATH
            }
            foreach ($line in $out) {
                if ($line) { Remove-Item -Path "Env:$line" -ErrorAction SilentlyContinue }
            }
            Remove-Item Env:CVM_AUTO -ErrorAction SilentlyContinue
            Remove-Item Env:CVM_AUTO_ROOT -ErrorAction SilentlyContinue
        }
        default {
            & $cvmBin @CvmArgs
        }
    }
}

function global:__cvm_auto_check {
    $currentPwd = $PWD.Path
    if ($global:CVM_AUTO_LAST_PWD -ceq $currentPwd) { return }
    $global:CVM_AUTO_LAST_PWD = $currentPwd

    if ($PWD.Provider.Name -ne 'FileSystem') {
        if ($env:CVM_AUTO -eq '1') { cvm deactivate }
        return
    }

    $dir = Get-Item -LiteralPath $currentPwd -ErrorAction SilentlyContinue
    $found = $null
    $name = $null
    while ($dir) {
        $cvmFile = Join-Path $dir.FullName '.cvm'
        if (Test-Path -LiteralPath $cvmFile -PathType Leaf) {
            $found = $dir.FullName
            $name = Get-Content -LiteralPath $cvmFile |
                ForEach-Object { $_.Trim() } |
                Where-Object { $_ -and -not $_.StartsWith('#') } |
                Select-Object -First 1
            break
        }
        $dir = $dir.Parent
    }

    if ($name) {
        if ($env:CVM_ENV -cne $name) {
            cvm use $name
            if ($env:CVM_ENV -cne $name) { return }
            $env:CVM_AUTO = '1'
            $env:CVM_AUTO_ROOT = $found
        }
    } elseif ($env:CVM_AUTO -eq '1') {
        cvm deactivate
        Remove-Item Env:CVM_AUTO -ErrorAction SilentlyContinue
        Remove-Item Env:CVM_AUTO_ROOT -ErrorAction SilentlyContinue
    }
}

if (-not (Test-Path variable:global:CVM_OLD_PROMPT)) {
    $global:CVM_OLD_PROMPT = (Get-Content function:prompt).ToString()
}
function global:prompt {
    __cvm_auto_check
    $prefix = if ($env:CVM_ENV) { "($env:CVM_ENV) " } else { '' }
    $prefix + (& ([scriptblock]::Create($global:CVM_OLD_PROMPT)))
}

__cvm_auto_check
"#;
