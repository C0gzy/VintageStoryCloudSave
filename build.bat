@echo off
REM Build script for Windows
REM Usage: build.bat [release|debug]

setlocal

set BUILD_TYPE=%1
if "%BUILD_TYPE%"=="" set BUILD_TYPE=release

echo Building for Windows (%BUILD_TYPE%)...

cd cloud-save-uploader

if "%BUILD_TYPE%"=="release" (
    cargo build --release
    echo.
    echo Build complete!
    echo Binary location: target\release\cloud-save-uploader.exe
) else (
    cargo build
    echo.
    echo Build complete!
    echo Binary location: target\debug\cloud-save-uploader.exe
)

endlocal

