param(
    [int]$EmployeeCount = 10000,
    [string]$OutputDir = "fixtures/scale/generated"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$resolvedOutput = Join-Path $root $OutputDir
New-Item -ItemType Directory -Force -Path $resolvedOutput | Out-Null

$employeesPath = Join-Path $resolvedOutput "employees.csv"
$attendancePath = Join-Path $resolvedOutput "attendance.csv"
$departments = @("operations", "sales", "backoffice")

$employeeLines = New-Object System.Collections.Generic.List[string]
$attendanceLines = New-Object System.Collections.Generic.List[string]
$employeeLines.Add("社員ID,氏名,部署,入社日,雇用状態")
$attendanceLines.Add("社員ID,勤務日,出勤時刻,退勤時刻,休憩分")

for ($i = 1; $i -le $EmployeeCount; $i++) {
    $employeeId = "E{0:D5}" -f $i
    $department = $departments[($i + 20260603) % $departments.Count]
    $employeeLines.Add("$employeeId,合成従業員$i,$department,2024-04-01,在籍")
    $attendanceLines.Add("$employeeId,2026-01-05,09:00,18:00,60")
}

Set-Content -Path $employeesPath -Value $employeeLines -Encoding UTF8
Set-Content -Path $attendancePath -Value $attendanceLines -Encoding UTF8

Write-Output "generated employees=$EmployeeCount output=$resolvedOutput"
