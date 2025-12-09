@echo off
echo ========================================
echo Building RustCOM Release Package
echo ========================================
echo.

REM Clean previous builds
echo [1/5] Cleaning previous builds...
cargo clean
if errorlevel 1 goto error

REM Build release version
echo [2/5] Building optimized release...
cargo build --release
if errorlevel 1 goto error

REM Create release folder
echo [3/5] Creating release package...
if exist release rmdir /s /q release
mkdir release
if errorlevel 1 goto error

REM Copy files
echo [4/5] Copying files...
copy target\release\rustcom.exe release\
copy README.md release\
copy FEATURES.md release\
copy COM0COM_SETUP.md release\
copy LICENSE release\

REM Create version info
echo [5/5] Creating version info...
echo RustCOM - Professional COM Port Analyzer > release\VERSION.txt
echo Version: 1.0.0 >> release\VERSION.txt
echo Build Date: %date% %time% >> release\VERSION.txt
echo. >> release\VERSION.txt
echo For support and updates: >> release\VERSION.txt
echo https://github.com/yourusername/rustcom >> release\VERSION.txt

echo.
echo ========================================
echo ✓ Build Complete!
echo ========================================
echo.
echo Release package created in: .\release\
echo.
echo Distribution files:
dir release /b
echo.
echo You can now:
echo 1. Test: release\rustcom.exe
echo 2. Zip the 'release' folder for distribution
echo 3. Create an installer using Inno Setup
echo.
pause
goto end

:error
echo.
echo ✗ Build failed!
echo.
pause
exit /b 1

:end

