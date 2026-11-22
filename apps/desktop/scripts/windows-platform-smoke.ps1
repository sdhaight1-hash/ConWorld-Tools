param(
    [Parameter(Mandatory = $true)][string] $Binary,
    [switch] $RendererBenchmark,
    [switch] $Ontology,
    [string] $CaptureDirectory,
    [switch] $RequireGpu
)

$ErrorActionPreference = "Stop"
$exe = (Resolve-Path $Binary).Path
if ($RequireGpu -and !$RendererBenchmark) { throw "-RequireGpu needs -RendererBenchmark." }
if ($Ontology -and $RendererBenchmark) { throw "Choose one smoke mode." }
if ($CaptureDirectory -and !$Ontology) { throw "-CaptureDirectory needs -Ontology." }
$arguments = @("--platform-smoke")
if ($Ontology) { $arguments = @("--ontology-smoke") }
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
$handshake = $null
if ($CaptureDirectory) {
    $CaptureDirectory = (New-Item -ItemType Directory -Force -Path $CaptureDirectory).FullName
    $handshake = (New-Item -ItemType Directory -Path (Join-Path ([System.IO.Path]::GetTempPath()) ([Guid]::NewGuid().ToString()))).FullName
    $startInfo.EnvironmentVariables["ATLAS_ONTOLOGY_CAPTURE_HANDSHAKE"] = $handshake
}
$process = New-Object System.Diagnostics.Process
$process.StartInfo = $startInfo
$started = $false
try {
    if (!$process.Start()) { throw "Could not launch the native platform test." }
    $started = $true
    $stdout = $process.StandardOutput.ReadToEndAsync()
    $stderr = $process.StandardError.ReadToEndAsync()
    if ($handshake) {
        $deadline = [DateTime]::UtcNow.AddSeconds(55)
        while (!(Test-Path (Join-Path $handshake "ready")) -and !$process.HasExited -and [DateTime]::UtcNow -lt $deadline) {
            Start-Sleep -Milliseconds 50
        }
        if (Test-Path (Join-Path $handshake "ready")) {
            Add-Type -AssemblyName System.Drawing
            Add-Type -AssemblyName System.Windows.Forms
            Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class AtlasNativeCapture {
    [StructLayout(LayoutKind.Sequential)]
    public struct Rect { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hwnd, out Rect rect);
    [DllImport("user32.dll")]
    public static extern bool MoveWindow(IntPtr hwnd, int x, int y, int width, int height, bool repaint);
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")]
    public static extern bool SetProcessDPIAware();
}
'@
            [void][AtlasNativeCapture]::SetProcessDPIAware()
            $process.Refresh()
            $hwnd = $process.MainWindowHandle
            if ($hwnd -eq [IntPtr]::Zero) { throw "Native ontology window is unavailable for capture." }
            [void][AtlasNativeCapture]::SetForegroundWindow($hwnd)
            Start-Sleep -Milliseconds 300
            if ([AtlasNativeCapture]::GetForegroundWindow() -ne $hwnd) { throw "Native ontology window is not foreground; refusing to capture another window." }
            $rect = New-Object AtlasNativeCapture+Rect
            if (![AtlasNativeCapture]::GetWindowRect($hwnd, [ref]$rect)) { throw "Cannot obtain native window bounds." }
            $width = $rect.Right - $rect.Left
            $height = $rect.Bottom - $rect.Top
            $bounds = New-Object System.Drawing.Rectangle($rect.Left, $rect.Top, $width, $height)
            $screen = [System.Windows.Forms.Screen]::FromHandle($hwnd)
            Write-Output "ATLAS_CAPTURE_BOUNDS initial=$bounds monitor=$($screen.Bounds) work=$($screen.WorkingArea)"
            # The default 1080x760 client area can exceed a small hosted desktop.
            # Fit the real window, never crop the screenshot or waive visibility.
            if (!$screen.Bounds.Contains($bounds)) {
                $work = $screen.WorkingArea
                $fitWidth = [Math]::Min($width, $work.Width)
                $fitHeight = [Math]::Min($height, $work.Height)
                if ($fitWidth -lt 460 -or $fitHeight -lt 540) {
                    throw "Capture desktop cannot accommodate the minimum native window size."
                }
                $x = [Math]::Max($work.Left, [Math]::Min($rect.Left, $work.Right - $fitWidth))
                $y = [Math]::Max($work.Top, [Math]::Min($rect.Top, $work.Bottom - $fitHeight))
                if (![AtlasNativeCapture]::MoveWindow($hwnd, $x, $y, $fitWidth, $fitHeight, $true)) {
                    throw "Cannot fit the native ontology window to the capture desktop."
                }
                Start-Sleep -Milliseconds 500
                if (![AtlasNativeCapture]::GetWindowRect($hwnd, [ref]$rect)) { throw "Cannot obtain fitted native window bounds." }
                $width = $rect.Right - $rect.Left
                $height = $rect.Bottom - $rect.Top
                $bounds = New-Object System.Drawing.Rectangle($rect.Left, $rect.Top, $width, $height)
                $screen = [System.Windows.Forms.Screen]::FromHandle($hwnd)
            }
            Write-Output "ATLAS_CAPTURE_BOUNDS final=$bounds monitor=$($screen.Bounds)"
            if ($width -lt 460 -or $height -lt 540 -or !$screen.Bounds.Contains($bounds) -or
                [AtlasNativeCapture]::GetForegroundWindow() -ne $hwnd) {
                throw "Native window is not fully visible on the capture desktop."
            }
            $bitmap = New-Object System.Drawing.Bitmap($width, $height)
            try {
                $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
                try {
                    $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
                } finally { $graphics.Dispose() }
                $stamp = [DateTime]::UtcNow.ToString("yyyy-MM-dd_HH-mm-ss'Z'")
                $capture = Join-Path $CaptureDirectory ($stamp + "_phase-03_semantic_native-ontology-evidence_01.png")
                $bitmap.Save($capture, [System.Drawing.Imaging.ImageFormat]::Png)
                Write-Output "ATLAS_NATIVE_CAPTURE $capture"
            } finally { $bitmap.Dispose() }
            Set-Content -Encoding ascii -Path (Join-Path $handshake "captured") -Value "captured"
        } else {
            throw "Native ontology capture readiness was not acknowledged."
        }
    }
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
    } elseif ($Ontology) {
        if (!$report.observation.ontology_verified) { throw "Native ontology authoring verification failed." }
    } elseif (!$report.observation.scene_verified -or !$report.observation.canvas2d_verified -or
              !$report.observation.history_verified) {
        throw "Windows scene, Canvas2D or history verification failed."
    }
} finally {
    if ($started -and !$process.HasExited) {
        try { $process.Kill(); $process.WaitForExit() } catch { }
    }
    $process.Dispose()
    if ($handshake) { Remove-Item -LiteralPath $handshake -Recurse -Force }
}
