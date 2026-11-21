param(
    [Parameter(Mandatory = $true)][string] $Binary,
    [switch] $RendererBenchmark,
    [switch] $RequireGpu
)

$ErrorActionPreference = "Stop"
$exe = (Resolve-Path $Binary).Path
if ($RequireGpu -and !$RendererBenchmark) { throw "-RequireGpu needs -RendererBenchmark." }
$arguments = @("--platform-smoke")
if ($RendererBenchmark) {
    $arguments = @("--renderer-smoke")
    if ($RequireGpu) { $arguments += "--require-gpu" }
    Get-CimInstance Win32_VideoController |
        Select-Object Name, DriverVersion |
        ConvertTo-Json -Compress | ForEach-Object { Write-Output "ATLAS_VIDEO_CONTROLLERS $_" }
}
# Own the process handle so its ExitCode stays readable in Windows PowerShell.
$startInfo = New-Object System.Diagnostics.ProcessStartInfo
$startInfo.FileName = $exe
$startInfo.Arguments = $arguments -join " "
$startInfo.UseShellExecute = $false
$startInfo.CreateNoWindow = $true
$startInfo.RedirectStandardOutput = $true
$startInfo.RedirectStandardError = $true
$process = New-Object System.Diagnostics.Process
$process.StartInfo = $startInfo
try {
    if (!$process.Start()) { throw "Could not launch the native platform test." }
    $stdout = $process.StandardOutput.ReadToEndAsync()
    $stderr = $process.StandardError.ReadToEndAsync()
    $finished = $process.WaitForExit(60000)
    if (!$finished) {
        try {
            $process.Kill()
        } catch [System.InvalidOperationException] {
            if (!$process.HasExited) { throw }
        }
        $process.WaitForExit()
    }
    if (![System.Threading.Tasks.Task]::WaitAll([System.Threading.Tasks.Task[]]@($stdout, $stderr), 5000)) {
        throw "The native platform test did not close its output streams."
    }
    $lines = $stdout.GetAwaiter().GetResult() -split '\r?\n'
    $lines | Write-Output
    $stderr.GetAwaiter().GetResult() | Write-Output
    if (!$finished) { throw "The embedded WebView did not complete its platform smoke test in 60 seconds." }
    $reports = @($lines | Where-Object { $_.StartsWith("ATLAS_PLATFORM_SMOKE ") })
    if ($reports.Count -ne 1) { throw "Expected one native platform smoke report." }
    $report = $reports[0].Substring("ATLAS_PLATFORM_SMOKE ".Length) | ConvertFrom-Json
    if ($process.ExitCode -ne 0 -or $report.target -ne "windows" -or !$report.webview_version) {
        throw "Windows platform verification failed (exit $($process.ExitCode))."
    }
    if ($RendererBenchmark) {
        if (!$report.observation.benchmark) { throw "Expected a native renderer benchmark report." }
    } elseif (!$report.observation.scene_verified -or !$report.observation.canvas2d_verified -or
              !$report.observation.history_verified) {
        throw "Windows scene, Canvas2D or history verification failed."
    }
} finally {
    $process.Dispose()
}
