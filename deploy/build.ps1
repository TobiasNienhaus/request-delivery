Start-Job -ScriptBlock {
    Set-Location $(Join-Path -Path $using:PWD -ChildPath "../backend")
    wsl cargo build --release -p server --target-dir="../deploy/target/rust" --timings --target x86_64-unknown-linux-gnu
}

Start-Job -ScriptBlock {
    Set-Location $(Join-Path -Path $using:PWD -ChildPath "../frontend")
    pnpm run build-prod --output-path "../deploy/target/frontend"
}

Get-Job | Wait-Job
Get-Job | Receive-Job
