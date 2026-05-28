
function Invoke-NpmWebpackWorkflow {
    param(
        [Parameter(Mandatory=$true)]
        $Config,
        [Parameter(Mandatory=$true)]
        $ScriptRoot,
        [Parameter(Mandatory=$true)]
        $OutputDir
    )

    Write-Host "`n--- Building $($Config.name) (Workflow: $($Config.workflow)) ---" -ForegroundColor Cyan
    
    Push-Location $Config.SourceDir
    try {
        # 1. Install dependencies
        if (-not (Test-Path "node_modules")) {
            Write-Host "[$($Config.name)] Installing npm dependencies..." -ForegroundColor Gray
            if (Test-Path "package-lock.json") {
                & npm ci 2>&1 | ForEach-Object { Write-Host "[$($Config.name)] $_" }
            } else {
                & npm install 2>&1 | ForEach-Object { Write-Host "[$($Config.name)] $_" }
            }
            if ($LASTEXITCODE -ne 0) {
                throw "npm install failed with exit code $LASTEXITCODE"
            }
        }

        # 2. Cleanup DistDir
        if (Test-Path $Config.distDir) {
            Write-Host "[$($Config.name)] Cleaning existing $($Config.distDir) directory..." -ForegroundColor Gray
            Remove-Item -Recurse -Force $Config.distDir
        }

        # 3. Execute build command
        Write-Host "[$($Config.name)] Running build: $($Config.buildCmd)" -ForegroundColor Gray
        # Use Start-Process to capture output better if needed, but Invoke-Expression is fine for now
        # but let's make it more robust
        & cmd /c "$($Config.buildCmd) 2>&1" | ForEach-Object { Write-Host "[$($Config.name)] $_" }
        if ($LASTEXITCODE -ne 0) {
            throw "Build command failed with exit code $LASTEXITCODE"
        }

        # 4. Verify output
        if (-not (Test-Path $Config.distDir)) {
            Write-Error "[$($Config.name)] Build finished but output directory not found: $($Config.distDir)"
            return $false
        }

        # 5. Copy to docs/project-name
        $TargetSubDir = Join-Path $ScriptRoot "$OutputDir/$($Config.name)"
        if (-not (Test-Path $TargetSubDir)) {
            New-Item -ItemType Directory -Path $TargetSubDir -Force | Out-Null
        } else {
            Remove-Item -Recurse -Force "$TargetSubDir/*"
        }

        Write-Host "[$($Config.name)] Copying artifacts to $TargetSubDir..." -ForegroundColor Gray
        Copy-Item -Path "$($Config.distDir)/*" -Destination $TargetSubDir -Recurse -Force
        return $true
    } catch {
        Write-Error "Failed to build project $($Config.name): $_"
        return $false
    } finally {
        Pop-Location
    }
}

function Invoke-Workflow {
    param(
        [Parameter(Mandatory=$true)]
        $Config,
        [Parameter(Mandatory=$true)]
        $ScriptRoot,
        [Parameter(Mandatory=$true)]
        $OutputDir
    )

    if ($Config.workflow -eq "npm-webpack") {
        return Invoke-NpmWebpackWorkflow -Config $Config -ScriptRoot $ScriptRoot -OutputDir $OutputDir
    } else {
        Write-Error "[$($Config.name)] Unknown workflow: $($Config.workflow)"
        return $false
    }
}

Export-ModuleMember -Function Invoke-Workflow
