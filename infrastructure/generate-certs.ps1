# Generate self-signed TLS certificates for OTEL Collector development (Windows)
# Requires: OpenSSL for Windows (https://slproweb.com/products/Win32OpenSSL.html)
# Usage: .\generate-certs.ps1
# Output: infrastructure\certs\ directory with CA, server, and client certs

$ErrorActionPreference = "Stop"

$CertsDir = Join-Path $PSScriptRoot "certs"
New-Item -ItemType Directory -Force -Path $CertsDir | Out-Null

Push-Location $CertsDir

Write-Host "==> Generating CA certificate..." -ForegroundColor Green
openssl req -new -x509 -days 3650 -nodes `
  -subj "/CN=HoneyLink Root CA/O=HoneyLink Dev/C=US" `
  -keyout ca.key `
  -out ca.crt

Write-Host "==> Generating server certificate..." -ForegroundColor Green
openssl req -new -nodes `
  -subj "/CN=otel-collector/O=HoneyLink Dev/C=US" `
  -keyout server.key `
  -out server.csr

# Server certificate SAN (Subject Alternative Names) for localhost and Docker
@"
subjectAltName = DNS:otel-collector,DNS:localhost,IP:127.0.0.1,IP:172.28.0.2
extendedKeyUsage = serverAuth
"@ | Out-File -Encoding ASCII server-ext.cnf

openssl x509 -req -in server.csr -days 365 `
  -CA ca.crt -CAkey ca.key -CAcreateserial `
  -out server.crt `
  -extfile server-ext.cnf

Write-Host "==> Generating client certificate..." -ForegroundColor Green
openssl req -new -nodes `
  -subj "/CN=honeylink-telemetry/O=HoneyLink Dev/C=US" `
  -keyout client.key `
  -out client.csr

@"
extendedKeyUsage = clientAuth
"@ | Out-File -Encoding ASCII client-ext.cnf

openssl x509 -req -in client.csr -days 365 `
  -CA ca.crt -CAkey ca.key -CAcreateserial `
  -out client.crt `
  -extfile client-ext.cnf

# Cleanup temporary files
Remove-Item -Force *.csr, *.cnf, *.srl -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "==> Certificates generated successfully:" -ForegroundColor Green
Write-Host "    CA Certificate:     $CertsDir\ca.crt"
Write-Host "    Server Certificate: $CertsDir\server.crt"
Write-Host "    Server Key:         $CertsDir\server.key"
Write-Host "    Client Certificate: $CertsDir\client.crt"
Write-Host "    Client Key:         $CertsDir\client.key"
Write-Host ""
Write-Host "==> Certificate details:" -ForegroundColor Cyan
openssl x509 -in server.crt -noout -text | Select-String -Pattern "Subject Alternative Name" -Context 0,2
Write-Host ""
Write-Host "==> Next steps:" -ForegroundColor Yellow
Write-Host "    1. Copy ca.crt to honeylink-telemetry crate config"
Write-Host "    2. Run: docker-compose -f infrastructure\docker-compose.observability.yml up -d"
Write-Host "    3. Verify OTLP endpoint: Invoke-WebRequest -Uri https://localhost:4317 -SkipCertificateCheck"

Pop-Location
