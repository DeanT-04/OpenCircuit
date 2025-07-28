# 🚀 OpenCircuit Setup Guide - Windows Development Environment

## 🎯 Prerequisites

This guide will set up a complete development environment for OpenCircuit on Windows using PowerShell and modern tooling.

## 📦 Step 1: Install Core Tools

### Install Chocolatey (Package Manager)
Open PowerShell as Administrator and run:
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
```

### Install Development Tools
```powershell
# Core development tools
choco install git -y
choco install vscode -y
choco install nodejs -y
choco install python -y

# Rust toolchain
choco install rustup.install -y

# Additional utilities
choco install 7zip -y
choco install wget -y
```

## 🦀 Step 2: Rust Development Setup

### Initialize Rust Toolchain
```powershell
# Refresh environment variables
refreshenv

# Install stable Rust toolchain
rustup default stable
rustup update

# Install additional components
rustup component add clippy
rustup component add rustfmt
rustup component add rust-analyzer

# Install useful cargo tools
cargo install cargo-watch
cargo install cargo-expand
cargo install cargo-audit
cargo install tauri-cli
```

### Verify Rust Installation
```powershell
rustc --version
cargo --version
```

## 🖥️ Step 3: Tauri Prerequisites

### Install WebView2 Runtime
```powershell
# Download and install WebView2 Runtime
choco install webview2 -y
```

### Install Visual Studio Build Tools
```powershell
# Install MSVC build tools (required for native dependencies)
choco install visualstudio2022buildtools -y
choco install visualstudio2022-workload-vctools -y
```

## 🔧 Step 4: Project Setup

### Clone and Initialize Project
```powershell
# Navigate to your projects directory
cd "C:\Users\$env:USERNAME\Documents\Coding projects\projects\OpenCircuit\OpenCircuit"

# Initialize Cargo project (if not already done)
cargo init --name opencircuit

# Create Tauri configuration
cargo tauri init
```

### Install Project Dependencies
```powershell
# Install Rust dependencies
cargo build

# Install Node.js dependencies (for Tauri frontend)
bun install
```

## 📁 Step 5: Project Structure Setup

### Create Core Directory Structure
```powershell
# Create source directories
New-Item -ItemType Directory -Force -Path "src\ai"
New-Item -ItemType Directory -Force -Path "src\circuit"
New-Item -ItemType Directory -Force -Path "src\pcb"
New-Item -ItemType Directory -Force -Path "src\gui"
New-Item -ItemType Directory -Force -Path "src\database"
New-Item -ItemType Directory -Force -Path "src\export"
New-Item -ItemType Directory -Force -Path "src\utils"

# Create data directories
New-Item -ItemType Directory -Force -Path "data\components"
New-Item -ItemType Directory -Force -Path "data\libraries"
New-Item -ItemType Directory -Force -Path "data\templates"

# Create test directories
New-Item -ItemType Directory -Force -Path "tests\integration"
New-Item -ItemType Directory -Force -Path "tests\fixtures"

# Create documentation directories (already exists)
# New-Item -ItemType Directory -Force -Path "docs"
# New-Item -ItemType Directory -Force -Path "project_docs"
```

### Initialize Configuration Files
```powershell
# Create .env file for development
@"
# OpenCircuit Development Configuration
RUST_LOG=debug
OPENAI_API_KEY=your_openai_api_key_here
ANTHROPIC_API_KEY=your_anthropic_api_key_here
DATABASE_URL=sqlite:data/opencircuit.db
"@ | Out-File -FilePath ".env" -Encoding UTF8

# Create .gitignore
@"
# Rust
/target/
Cargo.lock

# IDE
.vscode/settings.json
.idea/

# Environment
.env.local
.env.production

# Database
*.db
*.sqlite

# Logs
*.log

# OS
.DS_Store
Thumbs.db

# Tauri
/src-tauri/target/
/dist/
"@ | Out-File -FilePath ".gitignore" -Encoding UTF8
```

## 🔑 Step 6: API Keys Setup

### Configure Environment Variables
```powershell
# Create secure API key storage
$envFile = ".env"
Write-Host "Please add your API keys to the .env file:"
Write-Host "1. OpenAI API Key: https://platform.openai.com/api-keys"
Write-Host "2. Anthropic API Key: https://console.anthropic.com/"
Write-Host "3. Component API Keys (Octopart, DigiKey, etc.)"

# Open .env file for editing
code .env
```

## 🗄️ Step 7: Database Setup

### Initialize SQLite Database
```powershell
# Install SQLite
choco install sqlite -y

# Create database directory
New-Item -ItemType Directory -Force -Path "data"

# Initialize database (will be created by application)
Write-Host "Database will be automatically created on first run"
```

## 🧪 Step 8: Development Workflow

### Development Commands
```powershell
# Start development server with hot reload
cargo tauri dev

# Build for production
cargo tauri build

# Run tests
cargo test

# Run with file watching (auto-rebuild)
cargo watch -x "tauri dev"

# Format code
cargo fmt

# Lint code
cargo clippy

# Check for security vulnerabilities
cargo audit
```

### VS Code Setup
```powershell
# Install recommended VS Code extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension tauri-apps.tauri-vscode
code --install-extension ms-vscode.vscode-json
code --install-extension bradlc.vscode-tailwindcss

# Open project in VS Code
code .
```

## 🔧 Step 9: NgSpice Integration

### Install NgSpice
```powershell
# Download NgSpice for Windows
$ngspiceUrl = "https://sourceforge.net/projects/ngspice/files/ng-spice-rework/39/ngspice-39_64.zip"
$ngspiceZip = "ngspice-39_64.zip"
$ngspiceDir = "C:\ngspice"

# Download and extract
Invoke-WebRequest -Uri $ngspiceUrl -OutFile $ngspiceZip
Expand-Archive -Path $ngspiceZip -DestinationPath $ngspiceDir -Force
Remove-Item $ngspiceZip

# Add to PATH
$env:PATH += ";$ngspiceDir\Spice64\bin"
[Environment]::SetEnvironmentVariable("PATH", $env:PATH, [EnvironmentVariableTarget]::User)

Write-Host "NgSpice installed to $ngspiceDir"
```

## 🚀 Step 10: First Run

### Verify Installation
```powershell
# Check all tools are working
Write-Host "=== Verification ==="
Write-Host "Rust version:"
rustc --version

Write-Host "Cargo version:"
cargo --version

Write-Host "Node.js version:"
node --version

Write-Host "Tauri CLI version:"
cargo tauri --version

Write-Host "NgSpice version:"
& "$ngspiceDir\Spice64\bin\ngspice.exe" --version
```

### Start Development
```powershell
# Initialize Git repository
git init
git add .
git commit -m "Initial OpenCircuit project setup"

# Start development server
Write-Host "Starting OpenCircuit development server..."
cargo tauri dev
```

## 🔄 Step 11: Git Workflow

### Setup Git Configuration
```powershell
# Configure Git (replace with your details)
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# Create GitHub repository (optional)
# gh repo create opencircuit --public --description "AI-Powered PCB Design Tool"
# git remote add origin https://github.com/yourusername/opencircuit.git
# git push -u origin main
```

### Development Workflow
```powershell
# Daily development workflow
git pull                    # Get latest changes
cargo tauri dev            # Start development
# ... make changes ...
cargo test                 # Run tests
cargo fmt                  # Format code
cargo clippy               # Check for issues
git add .                  # Stage changes
git commit -m "feat: add new feature"  # Commit changes
git push                   # Push to remote
```

## 🛠️ Troubleshooting

### Common Issues

#### Rust Compilation Errors
```powershell
# Update Rust toolchain
rustup update

# Clean build cache
cargo clean
cargo build
```

#### Tauri Build Issues
```powershell
# Reinstall Tauri CLI
cargo install tauri-cli --force

# Check WebView2 installation
Get-AppxPackage -Name "Microsoft.WebView2"
```

#### NgSpice Integration Issues
```powershell
# Verify NgSpice installation
Test-Path "C:\ngspice\Spice64\bin\ngspice.exe"

# Check PATH environment variable
$env:PATH -split ";" | Where-Object { $_ -like "*ngspice*" }
```

## 📚 Next Steps

1. **Read Documentation**: Explore the `docs/` folder for technical details
2. **Review Tasks**: Check `project_docs/tasks.md` for development roadmap
3. **Start Coding**: Begin with the first task in the task list
4. **Join Community**: Contribute to the open-source project

## 🔗 Useful Resources

- **Rust Documentation**: https://doc.rust-lang.org/
- **Tauri Documentation**: https://tauri.app/
- **egui Documentation**: https://docs.rs/egui/
- **NgSpice Manual**: http://ngspice.sourceforge.net/docs.html
- **OpenCircuit Docs**: `docs/index.md`

---

*Setup Guide Version: 1.0*  
*Last Updated: 2025-01-27*  
*Platform: Windows 10/11*