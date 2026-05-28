
param(
    [switch]$Preview,
    [string]$ConfigFile,
    [switch]$Parallel
)

Set-StrictMode -Version Latest

# --- Configuration ---
$OutputDir = "docs"
$ExcludedDirs = @("node_modules", "target", ".git", $OutputDir, "pkg")
$RepoRoot = Split-Path $PSScriptRoot -Parent
$PublishInternalDir = $PSScriptRoot

# Import Workflows
Import-Module (Join-Path $PublishInternalDir "Workflows.psm1") -Force

# --- Helper Functions ---

function Get-ProjectConfigs {
    if ($ConfigFile) {
        if (Test-Path $ConfigFile) {
            $Configs = Get-Content $ConfigFile | ConvertFrom-Json
            # Ensure it's a list for consistent processing
            if ($Configs -isnot [array]) {
                $Configs = @($Configs)
            }
            # For manual config files, ensure SourceDir is absolute or relative to RepoRoot
            foreach ($Config in $Configs) {
                if (-not (Get-Member -InputObject $Config -Name "SourceDir")) {
                    Write-Warning "Project $($Config.name) missing SourceDir in config file."
                } elseif (-not [System.IO.Path]::IsPathRooted($Config.SourceDir)) {
                    $Config.SourceDir = [System.IO.Path]::GetFullPath((Join-Path $RepoRoot $Config.SourceDir))
                }
            }
            return $Configs
        } else {
            Write-Error "Config file not found: $ConfigFile"
            return @()
        }
    }

    Write-Host "Crawling for publish.json files..." -ForegroundColor Gray
    $Configs = [System.Collections.Generic.List[PSObject]]::new()
    
    function Search-Dir($Path, $ConfigsList) {
        $Items = Get-ChildItem -Path $Path -Force
        foreach ($Item in $Items) {
            if ($Item.PSIsContainer) {
                if ($ExcludedDirs -notcontains $Item.Name) {
                    Search-Dir $Item.FullName $ConfigsList
                }
            } elseif ($Item.Name -eq "publish.json") {
                $Config = Get-Content $Item.FullName | ConvertFrom-Json
                $Config | Add-Member -MemberType NoteProperty -Name "SourceDir" -Value $Item.DirectoryName
                $ConfigsList.Add($Config)
            }
        }
    }

    Search-Dir $RepoRoot $Configs
    return $Configs
}

# --- Execution ---

# 1. Prepare Output Directory
if (Test-Path $OutputDir) {
    Write-Host "Cleaning existing $OutputDir directory..." -ForegroundColor Gray
    Get-ChildItem -Path $OutputDir | Remove-Item -Recurse -Force
} else {
    New-Item -ItemType Directory -Path $OutputDir
}

# 2. Discover and Build Projects
$ProjectConfigs = Get-ProjectConfigs
$SuccessfulProjects = [System.Collections.Generic.List[PSObject]]::new()
$FailedProjects = [System.Collections.Generic.List[PSObject]]::new()

if ($Parallel -and $PSVersionTable.PSVersion.Major -ge 7) {
    Write-Host "`nBuilding projects in parallel..." -ForegroundColor Yellow
    $Results = $ProjectConfigs | ForEach-Object -Parallel {
        $RepoRoot = $using:RepoRoot
        $OutputDir = $using:OutputDir
        $WorkflowModule = Join-Path $using:PublishInternalDir "Workflows.psm1"
        
        Import-Module $WorkflowModule -Force
        
        # Ensure path is updated for child processes in parallel
        if ($env:USERPROFILE) {
            $CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
            if ($env:PATH -notlike "*$CargoBin*") {
                $env:PATH = "$CargoBin;$env:PATH"
            }
        }

        $Success = Invoke-Workflow -Config $_ -ScriptRoot $RepoRoot -OutputDir $OutputDir
        return [PSCustomObject]@{
            Config = $_
            Success = $Success
        }
    } -ThrottleLimit 4

    foreach ($Res in $Results) {
        if ($Res.Success) { $SuccessfulProjects.Add($Res.Config) }
        else { $FailedProjects.Add($Res.Config) }
    }
} else {
    if ($Parallel) { Write-Warning "Parallel builds require PowerShell 7+. Falling back to sequential." }
    foreach ($Config in $ProjectConfigs) {
        if (Invoke-Workflow -Config $Config -ScriptRoot $RepoRoot -OutputDir $OutputDir) {
            $SuccessfulProjects.Add($Config)
        } else {
            $FailedProjects.Add($Config)
        }
    }
}

# 3. Build Summary
Write-Host "`n--- Build Summary ---" -ForegroundColor White
$TotalCount = if ($ProjectConfigs) { @($ProjectConfigs).Count } else { 0 }
Write-Host "Total Projects: $TotalCount"
Write-Host "Succeeded:      $($SuccessfulProjects.Count)" -ForegroundColor Green
if ($FailedProjects.Count -gt 0) {
    Write-Host "Failed:         $($FailedProjects.Count)" -ForegroundColor Red
    foreach ($P in $FailedProjects) {
        Write-Host "  - $($P.name)" -ForegroundColor Red
    }
    exit 1
}

if ($SuccessfulProjects.Count -eq 0) {
    Write-Warning "No projects were successfully built. Overview page will not be updated."
    if ($Preview) { exit }
}

# 4. Generate Overview Page
Write-Host "`nGenerating overview page..." -ForegroundColor Green

# Copy static assets
$AssetsSource = Join-Path $PublishInternalDir "Assets"
Copy-Item (Join-Path $AssetsSource "style.css") (Join-Path $OutputDir "style.css") -Force
Copy-Item (Join-Path $AssetsSource "app.js") (Join-Path $OutputDir "app.js") -Force

# Generate Links HTML
$Links = foreach ($Proj in $SuccessfulProjects) {
    $ProjId = "proj-$($Proj.Name)"
    $ProjName = $Proj.Name
    $ProjDescription = $Proj.Description
    @"
            <li class="project-item" id="$ProjId" role="option" data-url="./$ProjName/" aria-labelledby="$ProjId-title" aria-describedby="$ProjId-desc">
                <div class="project-info">
                    <h2 class="project-title" id="$ProjId-title">$ProjName</h2>
                    <p class="project-desc" id="$ProjId-desc">$ProjDescription</p>
                </div>
            </li>
"@
}
$LinksHtml = $Links -join "`n"

# Load and populate Template
$TemplatePath = Join-Path $AssetsSource "index.html"
$IndexContent = Get-Content $TemplatePath -Raw

# Get version info (Tag or Commit)
$Version = "Local Build"
$GitTag = git describe --tags --match "web-v*" --exact-match 2>$null
if ($GitTag) {
    $Version = $GitTag.Trim()
} else {
    $GitSha = git rev-parse --short HEAD 2>$null
    if ($GitSha) {
        $Version = $GitSha.Trim()
    }
}

$IndexContent = $IndexContent.Replace("{{LINKS}}", $LinksHtml)
$IndexContent = $IndexContent.Replace("{{YEAR}}", (Get-Date -Format "yyyy"))
$IndexContent = $IndexContent.Replace("{{DATE}}", (Get-Date -Format "yyyy-MM-dd HH:mm:ss"))
$IndexContent = $IndexContent.Replace("{{VERSION}}", $Version)

$IndexContent | Set-Content -Path "$OutputDir/index.html" -Encoding UTF8

# 5. Preview
if ($Preview) {
    Write-Host "`nStarting local preview..." -ForegroundColor Yellow
    Write-Host "Press Ctrl+C to stop the server." -ForegroundColor Gray
    npx serve $OutputDir
} else {
    Write-Host "`nDone! Build artifacts are in the '$OutputDir' directory." -ForegroundColor Green
}

exit 0
