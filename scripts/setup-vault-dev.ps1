# HoneyLink Vault Development Setup Script (Windows PowerShell)
#
# This script sets up a local HashiCorp Vault instance for development.
# WARNING: This is for DEVELOPMENT ONLY. Never use -dev mode in production.
#
# Prerequisites:
#   - HashiCorp Vault CLI installed (https://www.vaultproject.io/downloads)
#   - PowerShell 5.1 or higher
#
# Usage:
#   .\scripts\setup-vault-dev.ps1 [-VaultVersion "1.15.0"] [-Port 8200]

[CmdletBinding()]
param(
    [Parameter()]
    [string]$VaultVersion = "1.15.0",

    [Parameter()]
    [int]$Port = 8200,

    [Parameter()]
    [switch]$SkipDownload
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# Color output functions
function Write-Success { Write-Host $args -ForegroundColor Green }
function Write-Info { Write-Host $args -ForegroundColor Cyan }
function Write-Warning { Write-Host $args -ForegroundColor Yellow }
function Write-ErrorMsg { Write-Host $args -ForegroundColor Red }

Write-Info "==> HoneyLink Vault Development Setup"
Write-Info "    Version: $VaultVersion"
Write-Info "    Port: $Port"

# Check if Vault is already installed
$vaultCmd = Get-Command vault -ErrorAction SilentlyContinue
if ($vaultCmd) {
    $installedVersion = & vault version | Select-String -Pattern "v\d+\.\d+\.\d+" | ForEach-Object { $_.Matches.Value }
    Write-Success "✓ Vault CLI found: $installedVersion"
}
elseif (-not $SkipDownload) {
    Write-Warning "! Vault CLI not found. Please install from: https://www.vaultproject.io/downloads"
    Write-Info "  Or use Chocolatey: choco install vault"
    exit 1
}

# Stop existing Vault dev server if running
$existingProcess = Get-Process -Name vault -ErrorAction SilentlyContinue
if ($existingProcess) {
    Write-Warning "! Stopping existing Vault process (PID: $($existingProcess.Id))"
    Stop-Process -Id $existingProcess.Id -Force
    Start-Sleep -Seconds 2
}

# Create data directory for dev mode persistence (optional)
$dataDir = Join-Path $PSScriptRoot ".." "data" "vault-dev"
if (-not (Test-Path $dataDir)) {
    New-Item -ItemType Directory -Path $dataDir -Force | Out-Null
    Write-Success "✓ Created data directory: $dataDir"
}

# Start Vault in dev mode with KV v2 enabled
Write-Info "==> Starting Vault dev server on port $Port..."
Write-Warning "! Development mode is INSECURE - root token: root"

$vaultArgs = @(
    "server",
    "-dev",
    "-dev-root-token-id=root",
    "-dev-listen-address=127.0.0.1:$Port"
)

# Start Vault as background job
$vaultJob = Start-Job -ScriptBlock {
    param($VaultPath, $Args)
    & $VaultPath $Args
} -ArgumentList (Get-Command vault).Source, $vaultArgs

Write-Success "✓ Vault server started (Job ID: $($vaultJob.Id))"
Write-Info "  To stop: Stop-Job -Id $($vaultJob.Id); Remove-Job -Id $($vaultJob.Id)"

# Wait for Vault to be ready
Write-Info "==> Waiting for Vault to be ready..."
$maxRetries = 10
$retryCount = 0
$vaultAddr = "http://127.0.0.1:$Port"

while ($retryCount -lt $maxRetries) {
    try {
        $response = Invoke-WebRequest -Uri "$vaultAddr/v1/sys/health" -Method GET -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            Write-Success "✓ Vault is ready"
            break
        }
    }
    catch {
        $retryCount++
        Start-Sleep -Seconds 1
    }
}

if ($retryCount -eq $maxRetries) {
    Write-ErrorMsg "✗ Vault failed to start after $maxRetries attempts"
    Stop-Job -Id $vaultJob.Id
    Remove-Job -Id $vaultJob.Id
    exit 1
}

# Configure environment variables
$env:VAULT_ADDR = $vaultAddr
$env:VAULT_TOKEN = "root"

Write-Info "==> Configuring Vault for HoneyLink key hierarchy..."

# Enable KV v2 secrets engine at honeylink/ path
Write-Info "  • Enabling KV v2 secrets engine..."
& vault secrets enable -path=honeylink kv-v2

# Create key hierarchy paths according to spec/security/encryption.md
$keyPaths = @(
    "honeylink/k_root",
    "honeylink/k_service",
    "honeylink/k_profile",
    "honeylink/k_telemetry"
)

Write-Info "  • Creating key hierarchy paths..."
foreach ($path in $keyPaths) {
    $pathName = ($path -split '/')[-1]
    Write-Info "    - $pathName"

    # Create initial placeholder (will be replaced by actual keys)
    $secretData = @{
        description = "HoneyLink $pathName hierarchy"
        created_at  = (Get-Date -Format "o")
        environment = "development"
        placeholder = $true
    } | ConvertTo-Json

    $null = & vault kv put $path @"
{
    "description": "HoneyLink $pathName hierarchy",
    "created_at": "$(Get-Date -Format 'o')",
    "environment": "development",
    "placeholder": true
}
"@
}

# Create policy for HoneyLink services
Write-Info "  • Creating access policy..."
$policyContent = @"
# HoneyLink Development Policy
# Allows full access to honeylink/* secrets for development

path "honeylink/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

path "honeylink/data/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

path "honeylink/metadata/*" {
  capabilities = ["list", "read", "delete"]
}

# Allow token renewal
path "auth/token/renew-self" {
  capabilities = ["update"]
}

# Allow token lookup
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
"@

$policyFile = Join-Path $env:TEMP "honeylink-dev-policy.hcl"
$policyContent | Out-File -FilePath $policyFile -Encoding UTF8
& vault policy write honeylink-dev $policyFile
Remove-Item $policyFile

Write-Success "`n✓ Vault development environment setup complete!"
Write-Info "`nQuick Start:"
Write-Info "  1. Set environment variables:"
Write-Info "     `$env:VAULT_ADDR = `"$vaultAddr`""
Write-Info "     `$env:VAULT_TOKEN = `"root`""
Write-Info "`n  2. Test connection:"
Write-Info "     vault status"
Write-Info "`n  3. View secrets:"
Write-Info "     vault kv list honeylink"
Write-Info "`n  4. Stop Vault:"
Write-Info "     Stop-Job -Id $($vaultJob.Id); Remove-Job -Id $($vaultJob.Id)"

Write-Warning "`n⚠ SECURITY REMINDER:"
Write-Warning "  • This is a DEVELOPMENT setup with root token 'root'"
Write-Warning "  • Data is stored in memory and will be lost on restart"
Write-Warning "  • Never use -dev mode or root tokens in production"
Write-Warning "  • For production setup, see docs/VAULT_SETUP.md"

# Save connection info to file for easy sourcing
$envFile = Join-Path $PSScriptRoot ".." ".vault-dev.env"
@"
# HoneyLink Vault Development Environment
# Source this file: . .\.vault-dev.env

`$env:VAULT_ADDR = "$vaultAddr"
`$env:VAULT_TOKEN = "root"
`$env:VAULT_NAMESPACE = ""
"@ | Out-File -FilePath $envFile -Encoding UTF8

Write-Success "✓ Environment variables saved to: $envFile"
