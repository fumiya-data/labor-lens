param(
    [string]$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
)

$ErrorActionPreference = "Stop"

function Join-RepoPath {
    param([string]$RelativePath)
    return Join-Path $Root $RelativePath
}

function Assert-Exists {
    param([string]$RelativePath)
    $path = Join-RepoPath $RelativePath
    if (-not (Test-Path $path)) {
        throw "Missing required path: $RelativePath"
    }
}

function Assert-FileContains {
    param(
        [string]$RelativePath,
        [string]$Pattern,
        [string]$Reason
    )
    $path = Join-RepoPath $RelativePath
    $content = Get-Content -Raw -Encoding UTF8 $path
    if ($content -notmatch $Pattern) {
        throw "$RelativePath does not contain expected policy: $Reason"
    }
}

function Assert-FileNotContains {
    param(
        [string]$RelativePath,
        [string]$Pattern,
        [string]$Reason
    )
    $path = Join-RepoPath $RelativePath
    $content = Get-Content -Raw -Encoding UTF8 $path
    if ($content -match $Pattern) {
        throw "$RelativePath still contains retired policy: $Reason"
    }
}

function Assert-NoIndependentCrateScaffolds {
    $cratesPath = Join-RepoPath "crates"
    if (-not (Test-Path $cratesPath)) {
        return
    }

    $retired = Get-ChildItem -Path $cratesPath -Directory -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like "laborlens-*" }
    if ($retired) {
        $names = ($retired | Select-Object -ExpandProperty Name) -join ", "
        throw "Retired independent crate scaffold directories still exist under crates/: $names"
    }
}

Assert-Exists "README.md"
Assert-Exists "docs\planning\REPOSITORY-PLAN.md"
Assert-Exists "docs\product\ARCHITECTURE.md"
Assert-Exists "docs\product\DATA-DESIGN.md"
Assert-Exists "docs\product\LEAN-SPEC-PLANNING.md"
Assert-Exists "lean\LaborLens\Spec\Privacy.lean"
Assert-Exists "lean\LaborLens\Spec\Workforce.lean"
Assert-Exists "lean\LaborLens\Spec\GuideSafety.lean"
Assert-Exists "lean\LaborLens\Theorems\PrivacyTheorems.lean"
Assert-Exists "lean\LaborLens\Theorems\WorkforceTheorems.lean"
Assert-Exists "lean\LaborLens\Theorems\GuideSafetyTheorems.lean"
Assert-Exists "reports\README.md"
Assert-Exists "tools\README.md"
Assert-Exists "tools\generate-scale-fixture.ps1"

Assert-Exists "Cargo.toml"
Assert-Exists "apps\laborlens-rust\Cargo.toml"
Assert-Exists "apps\laborlens-rust\README.md"
Assert-Exists "apps\laborlens-rust\src\main.rs"
Assert-Exists "apps\laborlens-rust\src\contexts\mod.rs"
Assert-Exists "apps\laborlens-rust\src\contexts\ingest\mod.rs"
Assert-Exists "apps\laborlens-rust\src\contexts\workforce_analysis\mod.rs"
Assert-Exists "apps\laborlens-rust\src\contexts\privacy_safety\mod.rs"
Assert-Exists "apps\laborlens-rust\src\contexts\reporting\mod.rs"
Assert-Exists "apps\laborlens-rust\src\contexts\guidance\mod.rs"
Assert-Exists "apps\laborlens-local-server\Cargo.toml"
Assert-Exists "apps\laborlens-local-server\src\lib.rs"
Assert-Exists "apps\laborlens-local-ui\index.html"
Assert-Exists "apps\laborlens-local-ui\src\app.js"
Assert-Exists "apps\laborlens-local-ui\src\styles.css"
Assert-Exists "fixtures\privacy\fatigue.csv"
Assert-Exists "fixtures\scale\scale-seed.json"

foreach ($context in @("ingest", "workforce_analysis", "privacy_safety", "reporting", "guidance")) {
    foreach ($module in @("domain", "application", "infrastructure", "interfaces")) {
        Assert-Exists "apps\laborlens-rust\src\contexts\$context\$module.rs"
    }
}

Assert-FileContains "docs\planning\REPOSITORY-PLAN.md" "modular monolith" "repository plan names modular monolith as the production structure"
Assert-FileContains "docs\planning\REPOSITORY-PLAN.md" "bounded context" "repository plan uses bounded contexts"
Assert-FileContains "docs\planning\REPOSITORY-PLAN.md" "ingest" "repository plan includes ingest context"
Assert-FileContains "docs\planning\REPOSITORY-PLAN.md" "workforce analysis" "repository plan includes workforce analysis context"
Assert-FileContains "docs\planning\REPOSITORY-PLAN.md" "privacy/safety" "repository plan includes privacy/safety context"
Assert-FileContains "docs\planning\REPOSITORY-PLAN.md" "reporting" "repository plan includes reporting context"
Assert-FileContains "docs\planning\REPOSITORY-PLAN.md" "guidance" "repository plan includes guidance context"
Assert-FileContains "docs\planning\REPOSITORY-PLAN.md" "apps/laborlens-rust" "repository plan points Radomil to the Rust monolith scaffold"
Assert-FileContains "docs\planning\REPOSITORY-PLAN.md" "reports/" "repository plan keeps Pike's report connection visible"
Assert-FileContains "docs\planning\REPOSITORY-PLAN.md" "lean/" "repository plan keeps Leonard's Lean work visible"

Assert-FileNotContains "docs\planning\REPOSITORY-PLAN.md" "## レイヤー責務" "layer responsibility table"
Assert-FileNotContains "docs\planning\REPOSITORY-PLAN.md" "crates/\s*\r?\n\s*laborlens-domain" "layered crates topology"
Assert-FileNotContains "docs\planning\REPOSITORY-PLAN.md" "domain crate" "domain crate dependency rule"
Assert-FileNotContains "docs\planning\REPOSITORY-PLAN.md" "laborlens-(domain|ingest|storage|engine|jobs|safety|report|observability|cli)" "retired independent laborlens crate names"

Assert-FileContains "README.md" "modular monolith" "README summarizes the production structure"
Assert-FileContains "README.md" "apps/laborlens-rust" "README points Rust implementers to the monolith app"
Assert-FileContains "README.md" "reports/" "README documents the report app/output connection"
Assert-FileContains "README.md" "Lean" "README keeps Lean work visible"

Assert-FileContains "docs\product\ARCHITECTURE.md" "modular monolith" "product architecture follows repository plan"
Assert-FileContains "docs\product\DATA-DESIGN.md" "Rust monolith" "data design avoids a layered Rust-core/post-processing reading"
Assert-FileContains "reports\README.md" "Python" "reports README explains Pike's Python report connection"
Assert-FileContains "tools\README.md" "apps/laborlens-rust" "tools README keeps production logic out of utility scripts"
Assert-FileContains "crates\README.md" "retired" "crates README marks independent crate scaffolds as retired"
Assert-FileContains "apps\laborlens-local-ui\src\app.js" "/api/runs" "local UI calls local server instead of reimplementing core logic"
Assert-FileContains "apps\laborlens-local-server\src\lib.rs" "run_ingest_workflow" "local server calls the Rust monolith workflow"

Assert-NoIndependentCrateScaffolds

Write-Host "Repository structure validation passed."
