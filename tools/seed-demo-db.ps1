param(
    [string]$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path,
    [string]$DatabaseUrl = $env:DATABASE_URL,
    [string]$HostName = $(if ($env:PGHOST) { $env:PGHOST } else { "127.0.0.1" }),
    [int]$Port = $(if ($env:PGPORT) { [int]$env:PGPORT } else { 5432 }),
    [string]$Database = $(if ($env:PGDATABASE) { $env:PGDATABASE } else { "postgres" }),
    [string]$User = $(if ($env:PGUSER) { $env:PGUSER } else { $env:USERNAME })
)

$ErrorActionPreference = "Stop"

function Invoke-PsqlFile {
    param([string]$SqlPath)

    if ($DatabaseUrl) {
        & psql -w $DatabaseUrl -f $SqlPath
    } else {
        & psql -w -h $HostName -p $Port -U $User -d $Database -f $SqlPath
    }

    if ($LASTEXITCODE -ne 0) {
        throw "psql failed for $SqlPath. Set DATABASE_URL or PGHOST/PGPORT/PGDATABASE/PGUSER/PGPASSWORD."
    }
}

function Invoke-PsqlScalar {
    param([string]$Sql)

    if ($DatabaseUrl) {
        & psql -w $DatabaseUrl -At -c $Sql
    } else {
        & psql -w -h $HostName -p $Port -U $User -d $Database -At -c $Sql
    }

    if ($LASTEXITCODE -ne 0) {
        throw "psql scalar query failed. Set DATABASE_URL or PGHOST/PGPORT/PGDATABASE/PGUSER/PGPASSWORD."
    }
}

$migration = Join-Path $Root "db\migrations\0001_initial_postgresql_schema.sql"
$seed = Join-Path $Root "db\seeds\0001_demo_japanese_employees.sql"

Invoke-PsqlFile $migration
Invoke-PsqlFile $seed

$count = Invoke-PsqlScalar "SELECT count(*) FROM laborlens.demo_employees WHERE seed_version = 'demo_japanese_employees.v1';"
if ([int]$count -ne 1000) {
    throw "Expected 1000 demo employees, got $count"
}

Write-Host "Seeded laborlens.demo_employees with 1000 fictional Japanese employee records."
