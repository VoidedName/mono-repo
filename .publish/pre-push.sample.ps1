#!/usr/bin/env pwsh

# Git pre-push hook to automate deployment based on commit messages or tags.

$Remote = $args[0]
$Url = $args[1]

$ShouldPublish = $false
$Reason = ""

# Read from stdin to get the refs being pushed
# Format: <local ref> <local sha> <remote ref> <remote sha>
while ($line = [Console]::In.ReadLine()) {
    $parts = $line -split ' '
    $local_ref = $parts[0]
    $local_sha = $parts[1]
    $remote_ref = $parts[2]

    # 1. Check for tags starting with web-v
    if ($local_ref -like "refs/tags/web-v*") {
        $ShouldPublish = $true
        $Reason = "Tag $($local_ref) detected"
        break
    }

    # 2. Check commit messages for [publish] in the range being pushed
    # If remote_sha is all zeros, it's a new branch
    $range = if ($parts[3] -eq ("0" * 40)) { $local_sha } else { "$($parts[3])..$local_sha" }
    
    $LogMessages = git log $range --pretty=format:%B
    if ($LogMessages -like "*[publish]*") {
        $ShouldPublish = $true
        $Reason = "Commit message with [publish] detected"
        break
    }
}

if ($ShouldPublish) {
    Write-Host "`n[Hook] $Reason. Triggering local deployment..." -ForegroundColor Cyan
    
    # Get the directory of this script to find Deploy.ps1
    # Note: .git/hooks/pre-push is where this script lives
    $DeployScript = Join-Path (git rev-parse --show-toplevel) ".publish\Deploy.ps1"
    
    if (Test-Path $DeployScript) {
        & pwsh -File $DeployScript
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Local deployment failed. Push aborted."
            exit 1
        }
    } else {
        Write-Warning "Deploy script not found at $DeployScript"
    }
}

exit 0
