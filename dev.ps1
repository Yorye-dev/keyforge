param([ValidateSet('run', 'check', 'test', 'build', 'clippy', 'fmt')][string]$Task = 'run')
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    # Use the isolated toolchain if present; otherwise use the user's Cargo.
    if (Test-Path '.tools\cargo\bin\cargo.exe') {
        $env:RUSTUP_HOME = Join-Path $PSScriptRoot '.tools\rustup'
        $env:CARGO_HOME = Join-Path $PSScriptRoot '.tools\cargo'
        $env:PATH = "$PSScriptRoot\.tools\cargo\bin;$PSScriptRoot\.tools\w64devkit\bin;$env:PATH"
        $env:LIBRARY_PATH = "$PSScriptRoot\.tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\lib\self-contained"
    }
    switch ($Task) {
        'fmt' { cargo fmt --check }
        'clippy' { cargo clippy --locked --all-targets -- -D warnings }
        default { cargo $Task --locked }
    }
    if ($LASTEXITCODE -ne 0) { throw "Cargo terminó con código $LASTEXITCODE" }
} finally { Pop-Location }
