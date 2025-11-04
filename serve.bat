@echo off
REM Serve script for Chess Engine WASM (Windows)

echo ======================================
echo Chess Engine - Local Development Server
echo ======================================
echo.

REM Check if pkg directory exists, if not build first
if not exist ".\pkg" (
    echo WASM module not found. Building first...
    echo.
    call build.bat
    if %ERRORLEVEL% NEQ 0 (
        exit /b 1
    )
)

set PORT=8080

REM Check for miniserve
where miniserve >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo Starting server with miniserve on http://localhost:%PORT%
    echo.
    echo Press Ctrl+C to stop the server
    echo.
    miniserve . -p %PORT% --index index.html
    exit /b 0
)

REM Check for Python 3
where python >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo Starting server with Python on http://localhost:%PORT%
    echo.
    echo Open your browser and navigate to:
    echo   http://localhost:%PORT%
    echo.
    echo Press Ctrl+C to stop the server
    echo.
    python -m http.server %PORT%
    exit /b 0
)

REM Check for Python (older installations)
where py >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo Starting server with Python on http://localhost:%PORT%
    echo.
    echo Open your browser and navigate to:
    echo   http://localhost:%PORT%
    echo.
    echo Press Ctrl+C to stop the server
    echo.
    py -m http.server %PORT%
    exit /b 0
)

echo Error: No suitable HTTP server found!
echo.
echo Please install one of the following:
echo   - miniserve: cargo install miniserve
echo   - Python 3: https://www.python.org/downloads/
echo.
echo Or use any other static file server and point it to this directory.
exit /b 1
