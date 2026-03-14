# Tender Checker - Windows Build Script
# Run: Right-click run-build.bat -> Run as Administrator

trap {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "  FATAL ERROR!" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host ""
    Write-Host "Error: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please send screenshot to developer" -ForegroundColor Yellow
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Tender Checker - Build Script v3" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check PowerShell version
Write-Host "[Info] PowerShell Version: $($PSVersionTable.PSVersion)" -ForegroundColor Gray
Write-Host "[Info] Current User: $env:USERNAME" -ForegroundColor Gray
Write-Host ""

# Check admin rights
Write-Host "Checking administrator privileges..." -ForegroundColor Yellow

try {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    $isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
} catch {
    Write-Host "[Error] Cannot check admin rights: $_" -ForegroundColor Red
    $isAdmin = $false
}

if (-not $isAdmin) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "  ERROR: Administrator Required!" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please follow these steps:" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Method 1:" -ForegroundColor White
    Write-Host "  1. Find run-build.bat" -ForegroundColor Gray
    Write-Host "  2. Right-click -> Run as Administrator" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Method 2:" -ForegroundColor White
    Write-Host "  1. Search 'PowerShell'" -ForegroundColor Gray
    Write-Host "  2. Right-click -> Run as Administrator" -ForegroundColor Gray
    Write-Host "  3. Type: cd project_folder" -ForegroundColor Gray
    Write-Host "  4. Type: .\build-windows.ps1" -ForegroundColor Gray
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host "[OK] Administrator privileges confirmed" -ForegroundColor Green
Write-Host ""

# Select install path
Write-Host "========================================" -ForegroundColor DarkGray
Write-Host "  Step 1/4: Configure Install Path" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor DarkGray
Write-Host ""
Write-Host "Rust and Cargo will be installed to this directory" -ForegroundColor Gray
Write-Host "Note: VS Build Tools can ONLY be installed to C: drive (Microsoft restriction)" -ForegroundColor Gray
Write-Host ""

$defaultPath = "C:\DevTools"
Write-Host "Default: $defaultPath" -ForegroundColor Cyan
Write-Host "Press Enter for default, or type path (e.g., D:\DevTools)" -ForegroundColor Gray
Write-Host ""

$InstallPath = Read-Host "Install path"

if ([string]::IsNullOrWhiteSpace($InstallPath)) {
    $InstallPath = $defaultPath
}

$InstallPath = $InstallPath.Trim('"').Trim("'")

Write-Host ""
Write-Host "Install to: $InstallPath" -ForegroundColor Cyan
Write-Host ""

# Create directories
$rustPath = Join-Path $InstallPath "rust"
$cargoPath = Join-Path $InstallPath "cargo"

try {
    if (-not (Test-Path $InstallPath)) {
        New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
        Write-Host "[OK] Directory created" -ForegroundColor Green
    } else {
        Write-Host "[OK] Directory exists" -ForegroundColor Green
    }
} catch {
    Write-Host "[Error] Cannot create directory: $_" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

# Set environment variables
Write-Host ""
Write-Host "Setting environment variables..." -ForegroundColor Yellow

try {
    $env:RUSTUP_HOME = $rustPath
    $env:CARGO_HOME = $cargoPath
    
    [Environment]::SetEnvironmentVariable("RUSTUP_HOME", $rustPath, "User")
    [Environment]::SetEnvironmentVariable("CARGO_HOME", $cargoPath, "User")
    
    $cargoBin = Join-Path $cargoPath "bin"
    $env:PATH = "$cargoBin;$env:PATH"
    
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if (-not $currentPath.Contains($cargoBin)) {
        [Environment]::SetEnvironmentVariable("PATH", "$cargoBin;$currentPath", "User")
    }
    
    Write-Host "[OK] Environment variables set" -ForegroundColor Green
} catch {
    Write-Host "[Error] Failed to set environment variables: $_" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host ""

# Check Node.js
Write-Host "========================================" -ForegroundColor DarkGray
Write-Host "  Step 2/4: Check Node.js" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor DarkGray
Write-Host ""

$nodeCmd = Get-Command node -ErrorAction SilentlyContinue
if ($nodeCmd) {
    try {
        $nodeVersion = & node -v 2>$null
        Write-Host "[OK] Node.js installed: $nodeVersion" -ForegroundColor Green
    } catch {
        Write-Host "[Warning] Node.js installed but cannot get version" -ForegroundColor Yellow
    }
} else {
    Write-Host "[!] Node.js NOT installed" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Please install Node.js:" -ForegroundColor Cyan
    Write-Host "  Download: https://nodejs.org/" -ForegroundColor White
    Write-Host "  Recommended: LTS version" -ForegroundColor Gray
    Write-Host ""
    Write-Host "After installation, run this script again" -ForegroundColor Yellow
    Read-Host "Press Enter to exit"
    exit 0
}

Write-Host ""

# Check Rust
Write-Host "========================================" -ForegroundColor DarkGray
Write-Host "  Step 3/4: Check Rust" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor DarkGray
Write-Host ""

# Check Rust in custom path
$cargoExe = Join-Path $cargoPath "bin\cargo.exe"
$rustcExe = Join-Path $cargoPath "bin\rustc.exe"

$hasRust = $false
$rustVersion = ""

if ((Test-Path $cargoExe) -and (Test-Path $rustcExe)) {
    $env:PATH = "$(Join-Path $cargoPath 'bin');$env:PATH"
    try {
        $rustVersion = & $rustcExe --version 2>$null
        $hasRust = $true
    } catch {}
}

if (-not $hasRust) {
    $rustCmd = Get-Command rustc -ErrorAction SilentlyContinue
    if ($rustCmd) {
        try {
            $rustVersion = & rustc --version 2>$null
            $hasRust = $true
        } catch {}
    }
}

if ($hasRust) {
    Write-Host "[OK] Rust installed: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "[!] Rust NOT installed, preparing to install..." -ForegroundColor Yellow
    Write-Host "    Target: $rustPath" -ForegroundColor Gray
    Write-Host ""
    
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupExe = Join-Path $env:TEMP "rustup-init.exe"
    
    Write-Host "Downloading Rust installer..." -ForegroundColor Yellow
    Write-Host "URL: $rustupUrl" -ForegroundColor Gray
    
    try {
        $downloaded = $false
        
        # Method 1: WebClient
        try {
            $webClient = New-Object System.Net.WebClient
            $webClient.DownloadFile($rustupUrl, $rustupExe)
            $downloaded = $true
            Write-Host "[OK] Download complete (WebClient)" -ForegroundColor Green
        } catch {
            Write-Host "[!] WebClient failed, trying alternative..." -ForegroundColor Yellow
        }
        
        # Method 2: Invoke-WebRequest
        if (-not $downloaded) {
            try {
                Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupExe -UseBasicParsing
                $downloaded = $true
                Write-Host "[OK] Download complete (Invoke-WebRequest)" -ForegroundColor Green
            } catch {
                Write-Host "[!] Invoke-WebRequest failed" -ForegroundColor Yellow
            }
        }
        
        if (-not $downloaded) {
            Write-Host ""
            Write-Host "[Error] Cannot download Rust installer" -ForegroundColor Red
            Write-Host ""
            Write-Host "Please install Rust manually:" -ForegroundColor Yellow
            Write-Host "  1. Visit https://rustup.rs/" -ForegroundColor White
            Write-Host "  2. Download and run rustup-init.exe" -ForegroundColor White
            Write-Host "  3. Run this script again after installation" -ForegroundColor White
            Write-Host ""
            Read-Host "Press Enter to exit"
            exit 1
        }
        
        Write-Host ""
        Write-Host "Installing Rust..." -ForegroundColor Yellow
        Write-Host "(Select option 1 for default installation)" -ForegroundColor Gray
        Write-Host ""
        
        $env:RUSTUP_HOME = $rustPath
        $env:CARGO_HOME = $cargoPath
        
        & $rustupExe -y --default-toolchain stable
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host ""
            Write-Host "[OK] Rust installed successfully!" -ForegroundColor Green
            Write-Host ""
            Write-Host "[!] Please close this window and run script again as Administrator" -ForegroundColor Yellow
            Read-Host "Press Enter to exit"
            exit 0
        } else {
            Write-Host "[Error] Rust installation failed, exit code: $LASTEXITCODE" -ForegroundColor Red
            Read-Host "Press Enter to exit"
            exit 1
        }
        
    } catch {
        Write-Host "[Error] Download or installation failed: $_" -ForegroundColor Red
        Read-Host "Press Enter to exit"
        exit 1
    }
}

Write-Host ""

# Check VS Build Tools
Write-Host "========================================" -ForegroundColor DarkGray
Write-Host "  Step 4/4: Check VS Build Tools" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor DarkGray
Write-Host ""

$hasVS = $false
$vsPath = ""

# Check vswhere
$vsWhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    try {
        $vsPath = & $vsWhere -latest -property installationPath 2>$null
        if ($vsPath -and $vsPath -ne "") {
            $hasVS = $true
            Write-Host "[OK] VS installed: $vsPath" -ForegroundColor Green
        }
    } catch {}
}

# Check vcvars64.bat
if (-not $hasVS) {
    $vcvarsPaths = @(
        (Join-Path ${env:ProgramFiles} "Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"),
        (Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"),
        (Join-Path ${env:ProgramFiles} "Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"),
        (Join-Path ${env:ProgramFiles} "Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat"),
        (Join-Path ${env:ProgramFiles} "Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat")
    )
    
    foreach ($p in $vcvarsPaths) {
        if (Test-Path $p) {
            $hasVS = $true
            Write-Host "[OK] VS Build Tools detected" -ForegroundColor Green
            break
        }
    }
}

if (-not $hasVS) {
    Write-Host "[!] VS Build Tools NOT installed" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "VS Build Tools is required to compile Tauri apps" -ForegroundColor Gray
    Write-Host "Note: Can ONLY install to C: drive, requires ~6GB" -ForegroundColor Gray
    Write-Host ""
    
    $installVS = Read-Host "Install VS Build Tools? (y/n)"
    
    if ($installVS -eq "y" -or $installVS -eq "Y") {
        Write-Host ""
        Write-Host "Installing VS Build Tools..." -ForegroundColor Yellow
        Write-Host "(This takes 15-30 minutes, please wait...)" -ForegroundColor Gray
        Write-Host ""
        
        try {
            winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended" --accept-package-agreements --accept-source-agreements
            
            Write-Host ""
            Write-Host "[OK] Installation complete!" -ForegroundColor Green
            Write-Host ""
            Write-Host "[!] Please close this window and run script again as Administrator" -ForegroundColor Yellow
            Read-Host "Press Enter to exit"
            exit 0
        } catch {
            Write-Host "[Error] Installation failed: $_" -ForegroundColor Red
            Write-Host ""
            Write-Host "Please install VS Build Tools manually:" -ForegroundColor Yellow
            Write-Host "  1. Visit https://visualstudio.microsoft.com/downloads/" -ForegroundColor White
            Write-Host "  2. Download 'Build Tools for Visual Studio 2022'" -ForegroundColor White
            Write-Host "  3. Select 'Desktop development with C++'" -ForegroundColor White
            Write-Host ""
            Read-Host "Press Enter to exit"
            exit 1
        }
    } else {
        Write-Host ""
        Write-Host "[!] VS Build Tools is required to compile Tauri apps" -ForegroundColor Yellow
        Write-Host ""
        Read-Host "Press Enter to exit"
        exit 0
    }
}

Write-Host ""

# Build project
Write-Host "========================================" -ForegroundColor DarkGray
Write-Host "  Building Project" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor DarkGray
Write-Host ""

# Get script directory
$scriptDir = $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($scriptDir)) {
    $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
}
if ([string]::IsNullOrWhiteSpace($scriptDir)) {
    $scriptDir = Get-Location
}

Write-Host "Project directory: $scriptDir" -ForegroundColor Gray
Set-Location $scriptDir

# Check package.json
if (-not (Test-Path "package.json")) {
    Write-Host "[Error] package.json not found" -ForegroundColor Red
    Write-Host "Please run script from project root directory" -ForegroundColor Yellow
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host ""

# Install npm dependencies
Write-Host "Installing npm dependencies..." -ForegroundColor Yellow
Write-Host ""

try {
    & npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "[Error] npm install failed, exit code: $LASTEXITCODE" -ForegroundColor Red
        Write-Host ""
        Write-Host "Possible causes:" -ForegroundColor Yellow
        Write-Host "  1. Network issue - check connection" -ForegroundColor White
        Write-Host "  2. npm registry - try: npm config set registry https://registry.npmmirror.com" -ForegroundColor White
        Write-Host "  3. Node.js version too old - upgrade to LTS" -ForegroundColor White
        Write-Host ""
        Read-Host "Press Enter to exit"
        exit 1
    }
} catch {
    Write-Host "[Error] npm install exception: $_" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host "[OK] npm dependencies installed" -ForegroundColor Green
Write-Host ""

# Build Tauri
Write-Host "Building Tauri application..." -ForegroundColor Yellow
Write-Host "First build takes 5-15 minutes, please wait..." -ForegroundColor Gray
Write-Host ""

try {
    & npm run tauri:build
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Red
        Write-Host "  Build Failed!" -ForegroundColor Red
        Write-Host "========================================" -ForegroundColor Red
        Write-Host ""
        Write-Host "Common issues:" -ForegroundColor Yellow
        Write-Host "  1. VS Build Tools not installed or incomplete" -ForegroundColor White
        Write-Host "  2. Rust version outdated - run: rustup update" -ForegroundColor White
        Write-Host "  3. Network issue - need to download Rust crates" -ForegroundColor White
        Write-Host ""
        Write-Host "Please check error messages above" -ForegroundColor Yellow
        Write-Host ""
        Read-Host "Press Enter to exit"
        exit 1
    }
} catch {
    Write-Host "[Error] Build exception: $_" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Build Success!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Output files:" -ForegroundColor Yellow
Write-Host "  EXE: $scriptDir\src-tauri\target\release\tender-checker.exe" -ForegroundColor White
Write-Host "  MSI: $scriptDir\src-tauri\target\release\bundle\msi\" -ForegroundColor White
Write-Host ""
Write-Host "Environment variables saved. Restart PowerShell to take effect." -ForegroundColor Gray
Write-Host ""

Read-Host "Press Enter to exit"
