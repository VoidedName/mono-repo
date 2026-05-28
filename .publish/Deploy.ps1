
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path $PSScriptRoot -Parent
$OutputDir = Join-Path $RepoRoot "docs"
$WorktreePath = Join-Path $RepoRoot ".publish\.gh-pages-worktree"
$Branch = "gh-pages"

function Clean-Up {
    if (Test-Path $WorktreePath) {
        Write-Host "Cleaning up temporary worktree..." -ForegroundColor Gray
        git worktree remove $WorktreePath -f
        if (Test-Path $WorktreePath) { Remove-Item -Recurse -Force $WorktreePath }
    }
}

try {
    # 1. Build Projects Locally
    Write-Host "--- Step 1: Building Projects ---" -ForegroundColor Cyan
    & "$PSScriptRoot\publish.ps1" -Parallel
    if ($LASTEXITCODE -ne 0) { throw "Build failed." }

    # 2. Prepare gh-pages branch
    Write-Host "`n--- Step 2: Preparing $Branch branch ---" -ForegroundColor Cyan
    
    # Check if branch exists
    $BranchExists = git branch --list $Branch
    if (-not $BranchExists) {
        Write-Host "Creating $Branch branch..." -ForegroundColor Gray
        git branch $Branch
    }

    # Remove existing worktree if it exists (safety)
    Clean-Up

    # Add worktree
    Write-Host "Creating worktree at $WorktreePath..." -ForegroundColor Gray
    git worktree add $WorktreePath $Branch

    # 3. Sync Files
    Write-Host "`n--- Step 3: Syncing artifacts ---" -ForegroundColor Cyan
    # Remove old content from the branch (except .git which isn't there in worktree root usually, but be safe)
    Get-ChildItem -Path $WorktreePath -Exclude ".git" | Remove-Item -Recurse -Force
    
    # Copy new content
    Copy-Item -Path "$OutputDir\*" -Destination $WorktreePath -Recurse -Force
    Write-Host "Artifacts copied to worktree." -ForegroundColor Gray

    # 4. Commit and Push
    Write-Host "`n--- Step 4: Committing and Pushing ---" -ForegroundColor Cyan
    Push-Location $WorktreePath
    
    $CurrentSha = git rev-parse --short HEAD
    $CommitMessage = "Publish site (from commit $CurrentSha)"
    
    git add .
    $Changes = git status --porcelain
    if (-not $Changes) {
        Write-Host "No changes to publish." -ForegroundColor Yellow
    } else {
        git commit -m $CommitMessage
        Write-Host "Pushing to origin $Branch..." -ForegroundColor Gray
        git push origin $Branch
        Write-Host "Successfully published!" -ForegroundColor Green
    }
    Pop-Location

} catch {
    Write-Error "Deployment failed: $_"
    exit 1
} finally {
    Clean-Up
}
