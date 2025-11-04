@echo off
REM Build script for Chess Engine WASM (Windows)

echo ======================================
echo Building Chess Engine for WASM...
echo ======================================
echo.

REM Check if wasm-pack is installed
where wasm-pack >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo Error: wasm-pack is not installed!
    echo Please install it with: cargo install wasm-pack
    exit /b 1
)

REM Check if wasm32-unknown-unknown target is installed
rustup target list | findstr /C:"wasm32-unknown-unknown (installed)" >nul
if %ERRORLEVEL% NEQ 0 (
    echo Error: wasm32-unknown-unknown target is not installed!
    echo Please install it with: rustup target add wasm32-unknown-unknown
    exit /b 1
)

REM Build the WASM module
echo Running wasm-pack build...
wasm-pack build --target web --release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo ======================================
    echo Build failed!
    echo ======================================
    exit /b 1
)

echo.
echo ======================================
echo Build completed successfully!
echo ======================================
echo.
echo Output directory: .\pkg
echo.
echo To run locally:
echo   serve.bat
echo.
echo Or manually start a server:
echo   python -m http.server 8080
echo   Then open: http://localhost:8080
echo.
