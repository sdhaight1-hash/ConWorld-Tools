@echo off
setlocal
title Living History Atlas

rem Always launch from the repository, even when double-clicked elsewhere.
pushd "%~dp0apps\desktop"
if errorlevel 1 (
    echo [ERROR] Could not find the apps\desktop directory.
    pause
    exit /b 1
)

where npm >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Node.js and npm must be installed and available on PATH.
    goto failed
)

where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Rust and Cargo must be installed and available on PATH.
    goto failed
)

if not exist "node_modules\.bin\tauri.cmd" (
    echo Installing desktop dependencies...
    call npm ci
    if errorlevel 1 goto failed
)

echo Starting Living History Atlas...
call npm run tauri dev
if errorlevel 1 goto failed

popd
exit /b 0

:failed
echo.
echo [ERROR] Living History Atlas could not start. See the messages above.
pause
popd
exit /b 1
