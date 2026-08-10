@echo off
REM Serve the IronRoot docsify site locally (Windows).
REM Tries docsify-cli first, then npx, then Python's http.server.

setlocal
set "DOCS_DIR=%~dp0"
if "%PORT%"=="" set "PORT=3000"

cd /d "%DOCS_DIR%"

echo Serving %DOCS_DIR% on http://localhost:%PORT%
echo.

where docsify >nul 2>&1
if %ERRORLEVEL%==0 (
    echo Using docsify-cli.
    docsify serve "%DOCS_DIR%" --port %PORT%
    goto :eof
)

where npx >nul 2>&1
if %ERRORLEVEL%==0 (
    npx --no-install docsify-cli --version >nul 2>&1
    if %ERRORLEVEL%==0 (
        echo Using docsify-cli via npx.
        npx --no-install docsify-cli serve "%DOCS_DIR%" --port %PORT%
        goto :eof
    )
)

where python >nul 2>&1
if %ERRORLEVEL%==0 (
    echo docsify-cli not found; falling back to python -m http.server.
    echo Tip: install live-reload with: npm i -g docsify-cli
    python -m http.server %PORT% --bind 127.0.0.1
    goto :eof
)

where py >nul 2>&1
if %ERRORLEVEL%==0 (
    echo docsify-cli not found; falling back to py -m http.server.
    py -m http.server %PORT% --bind 127.0.0.1
    goto :eof
)

echo ERROR: no server available.
echo Install either docsify-cli (npm i -g docsify-cli) or Python 3.
exit /b 1
