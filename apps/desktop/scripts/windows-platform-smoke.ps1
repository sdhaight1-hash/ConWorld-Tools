param([Parameter(Mandatory = $true)][string] $Binary)

$ErrorActionPreference = "Stop"
$exe = (Resolve-Path $Binary).Path
$stdout = New-TemporaryFile
$stderr = New-TemporaryFile
try {
    $process = Start-Process -FilePath $exe -ArgumentList "--platform-smoke" -NoNewWindow -PassThru `
        -RedirectStandardOutput $stdout.FullName -RedirectStandardError $stderr.FullName
    $finished = $process.WaitForExit(60000)
    if (!$finished) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        $process.WaitForExit()
    }
    $lines = @(Get-Content -Path $stdout.FullName)
    $lines | Write-Output
    Get-Content -Path $stderr.FullName | Write-Output
    if (!$finished) { throw "The embedded WebView did not complete its platform smoke test in 60 seconds." }
    $reports = @($lines | Where-Object { $_.StartsWith("ATLAS_PLATFORM_SMOKE ") })
    if ($reports.Count -ne 1) { throw "Expected one native platform smoke report." }
    $report = $reports[0].Substring("ATLAS_PLATFORM_SMOKE ".Length) | ConvertFrom-Json
    if ($process.ExitCode -ne 0 -or $report.target -ne "windows" -or !$report.webview_version -or
        !$report.observation.scene_verified -or !$report.observation.canvas2d_verified) {
        throw "Windows scene or Canvas2D verification failed (exit $($process.ExitCode))."
    }
} finally {
    Remove-Item $stdout.FullName, $stderr.FullName
}
