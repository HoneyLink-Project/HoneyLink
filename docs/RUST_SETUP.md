# Rust Development Environment Setup

**Badges:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> This document provides the complete setup procedure for the Rust development environment required for HoneyLink™ implementation. All tooling excludes C/C++ dependencies and uses pure Rust implementations.

---

## 1. System Requirements

### 1.1 Supported Platforms
- **Primary Development:** WSL2 (Ubuntu 22.04 LTS or later) on Windows 10/11
- **Alternative:** Windows 10/11 with Visual Studio 2022 Build Tools
- **CI/CD:** Linux x86_64 (GitHub Actions runners)

### 1.2 Prerequisites
**For WSL2 (Recommended):**
```bash
# Install WSL2 and Ubuntu if not already installed (Windows PowerShell as Administrator)
wsl --install -d Ubuntu-22.04
```

**For Windows Native:**
- Visual Studio 2022 Build Tools with "Desktop development with C++" workload (for Rust MSVC toolchain only, no C++ code will be written)
- Download from: https://visualstudio.microsoft.com/downloads/ (Build Tools for Visual Studio 2022)

---

## 2. Rust Toolchain Installation

### 2.1 Install Rust Stable

**WSL/Linux:**
```bash
# Install rustup (Rust toolchain manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Load Rust environment in current shell
source "$HOME/.cargo/env"

# Verify installation
rustc --version
cargo --version
rustup --version
```

**Windows (PowerShell):**
```powershell
# Download and run rustup-init.exe from https://rustup.rs/
# During installation, select "1) Proceed with standard installation (default - just press enter)"
# After installation, restart PowerShell and verify:
rustc --version
cargo --version
rustup --version
```

**Expected Output (as of 2025-10-01):**
```
rustc 1.89.0 (29483883e 2025-08-04)
cargo 1.89.0 (abcd1234 2025-08-01)
rustup 1.28.2 (e4f3ad6f8 2025-04-28)
```

### 2.2 Add Essential Components

**WSL/Linux and Windows:**
```bash
# Add clippy (linter) and rustfmt (code formatter)
rustup component add clippy rustfmt

# Verify components
rustup component list | grep -E "(clippy|rustfmt)"
```

**Expected Output:**
```
clippy-x86_64-unknown-linux-gnu (installed)
rustfmt-x86_64-unknown-linux-gnu (installed)
```

### 2.3 Install Cargo Extensions

**WSL/Linux:**
```bash
# Install coverage, audit, and license checking tools
cargo install cargo-llvm-cov cargo-audit cargo-deny

# Verify installations
cargo llvm-cov --version
cargo audit --version
cargo deny --version
```

**Windows (if Build Tools are installed):**
```powershell
# Install with locked dependencies to avoid version conflicts
cargo install cargo-llvm-cov cargo-audit cargo-deny --locked
```

**Expected Output:**
```
cargo-llvm-cov 0.6.19
cargo-audit 0.21.2
cargo-deny 0.18.4
```

**Troubleshooting (Windows Native):**
If you encounter linker errors (`lld-link.exe not found`), ensure Visual Studio Build Tools are installed:
1. Download Build Tools for Visual Studio 2022
2. Run installer and select "Desktop development with C++"
3. Restart PowerShell and retry `cargo install` commands

**Alternative for Windows Without Build Tools:**
Use WSL2 for development (recommended) or install GNU toolchain:
```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
```

---

## 3. Rust Version Management

### 3.1 Project Rust Version Specification

**File:** `rust-toolchain.toml` (to be created in project root)
```toml
[toolchain]
channel = "1.89.0"  # Lock to specific stable version
components = ["clippy", "rustfmt"]
targets = ["x86_64-unknown-linux-gnu", "wasm32-unknown-unknown"]
profile = "default"
```

### 3.2 Version Update Policy
- **Stable Channel:** Update to latest stable every quarter (Jan/Apr/Jul/Oct)
- **Security Updates:** Apply immediately if CVE affects Rust toolchain
- **Testing:** Validate all CI pipelines pass before committing version bump
- **Documentation:** Update this document with new version number

### 3.3 Version Pinning Rationale
- **Reproducibility:** Ensures all developers and CI use identical compiler version
- **Stability:** Avoids unexpected breakage from automatic updates
- **Audit Trail:** `rust-toolchain.toml` is version-controlled, changes are trackable

---

## 4. Code Quality Tools Configuration

### 4.1 Clippy (Linter)

**Configuration File:** `.clippy.toml` (project root)
```toml
# Enforce pedantic lints for high code quality
# Allow exceptions only with explicit justification comments

# Deny warnings in CI (treat all clippy warnings as errors)
# Enable this in CI pipeline, not in local development
# clippy::all, clippy::pedantic, clippy::cargo

# Allowed lints (with justification):
# clippy::module_name_repetitions - acceptable for domain clarity
# clippy::missing_errors_doc - covered by rustdoc examples
```

**Usage:**
```bash
# Run clippy locally (warnings only)
cargo clippy --all-targets --all-features

# Run clippy in CI mode (deny warnings)
cargo clippy --all-targets --all-features -- -D warnings

# Auto-fix safe lints
cargo clippy --fix --allow-dirty --allow-staged
```

### 4.2 Rustfmt (Code Formatter)

**Configuration File:** `rustfmt.toml` (project root)
```toml
# HoneyLink Rust formatting rules
# Aligned with Rust community conventions

edition = "2024"
max_width = 100
tab_spaces = 4
newline_style = "Unix"  # LF line endings
use_field_init_shorthand = true
use_try_shorthand = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
format_code_in_doc_comments = true
normalize_comments = true
wrap_comments = true
comment_width = 100
```

**Usage:**
```bash
# Format entire workspace
cargo fmt --all

# Check formatting without modifying files (CI mode)
cargo fmt --all -- --check
```

### 4.3 Cargo-llvm-cov (Code Coverage)

**Usage:**
```bash
# Generate coverage report in HTML format
cargo llvm-cov --all-features --workspace --html

# Open report in browser
# WSL: explorer.exe target/llvm-cov/html/index.html
# Linux: xdg-open target/llvm-cov/html/index.html

# Generate coverage for CI (lcov format for Codecov upload)
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Enforce minimum coverage threshold (80%)
cargo llvm-cov --all-features --workspace --fail-under-lines 80
```

**CI Integration:**
```yaml
# .github/workflows/coverage.yml
- name: Run coverage
  run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
- name: Upload to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: ./lcov.info
    fail_ci_if_error: true
```

### 4.4 Cargo-audit (Vulnerability Scanner)

**Usage:**
```bash
# Check for known vulnerabilities in dependencies
cargo audit

# Check with additional details
cargo audit --json | jq '.'

# Fail CI if any vulnerabilities found
cargo audit --deny warnings
```

**CI Integration:**
```yaml
# .github/workflows/security.yml
- name: Security audit
  run: cargo audit --deny warnings
```

**Auto-Update Configuration:**
- Run `cargo audit` daily in CI (scheduled workflow)
- Alert to Slack `#hl-sec-wg` channel if vulnerabilities found
- Fix within 48 hours for HIGH/CRITICAL severity

### 4.5 Cargo-deny (Dependency Policy Enforcement)

**Configuration File:** `deny.toml` (project root)
```toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"
ignore = []

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
]
deny = [
    "GPL-3.0",  # Copyleft license incompatible with proprietary use
    "AGPL-3.0",
]
copyleft = "deny"
allow-osi-fsf-free = "neither"
default = "deny"
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"  # Warn on duplicate dependencies
wildcards = "deny"  # Ban wildcard dependencies (e.g., "*")
highlight = "all"
workspace-default-features = "allow"
external-default-features = "allow"

# Deny C/C++ FFI dependencies
[[bans.deny]]
name = "openssl-sys"  # Use rustls instead
[[bans.deny]]
name = "libsqlite3-sys"  # Use rusqlite with bundled feature
[[bans.deny]]
name = "libz-sys"  # Use flate2 with miniz_oxide backend

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

**Usage:**
```bash
# Check all policies
cargo deny check

# Check specific policy
cargo deny check licenses
cargo deny check bans
cargo deny check advisories

# CI integration
cargo deny check --all-features
```

---

## 5. Development Workflow Integration

### 5.1 Pre-commit Hook Setup

**File:** `.git/hooks/pre-commit` (auto-generated by setup script)
```bash
#!/bin/bash
set -e

echo "Running Rust pre-commit checks..."

# Format check
echo "→ Checking code format..."
cargo fmt --all -- --check

# Clippy check
echo "→ Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Audit check (fast)
echo "→ Running cargo deny..."
cargo deny check bans licenses

echo "✓ All pre-commit checks passed!"
```

**Installation:**
```bash
# Make hook executable
chmod +x .git/hooks/pre-commit

# Or use husky for cross-platform hook management (Node.js required)
npm install --save-dev husky
npx husky install
npx husky add .git/hooks/pre-commit "cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings"
```

### 5.2 CI/CD Pipeline Integration

**File:** `.github/workflows/rust-ci.yml` (to be created)
```yaml
name: Rust CI

on:
  push:
    branches: [master, develop]
  pull_request:
    branches: [master, develop]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"  # Treat warnings as errors

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - name: Format check
        run: cargo fmt --all -- --check
      - name: Clippy check
        run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run tests
        run: cargo test --all-features --workspace

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: Swatinem/rust-cache@v2
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      - name: Generate coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
      - name: Upload to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: ./lcov.info
          fail_ci_if_error: true

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-audit
        run: cargo install cargo-audit
      - name: Security audit
        run: cargo audit --deny warnings
      - name: Install cargo-deny
        run: cargo install cargo-deny
      - name: Check dependencies
        run: cargo deny check
```

---

## 6. Rust Ecosystem Tools (No C/C++ Dependencies)

### 6.1 Verified Pure Rust Libraries

**Core Dependencies (all pure Rust):**
```toml
# Cargo.toml
[dependencies]
# Async runtime (no C/C++ dependencies)
tokio = { version = "1.47", features = ["full"] }

# Cryptography (pure Rust implementations)
x25519-dalek = "2.0"  # Key agreement
chacha20poly1305 = "0.10"  # AEAD encryption
hkdf = "0.13"  # Key derivation
sha2 = "0.11"  # SHA-256/512 hashing
ed25519-dalek = "2.1"  # Digital signatures
zeroize = "1.9"  # Secure memory clearing

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Web framework (pure Rust)
axum = "0.8"  # HTTP server
tower = "0.5"  # Middleware
hyper = { version = "1.5", features = ["full"] }

# TLS (pure Rust alternative to OpenSSL)
rustls = "0.23"
tokio-rustls = "0.26"
webpki-roots = "0.26"

# Database (pure Rust drivers)
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres"] }

# Observability (pure Rust OpenTelemetry)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
opentelemetry = "0.27"
opentelemetry-otlp = "0.27"

# Testing
proptest = "1.6"  # Property-based testing
criterion = "0.5"  # Benchmarking

[dev-dependencies]
wiremock = "0.6"  # HTTP mocking
```

### 6.2 Banned C/C++ Dependencies

**DO NOT USE (require C/C++ compilation):**
- `openssl` / `openssl-sys` → Use `rustls` instead
- `libsqlite3-sys` (unless bundled feature) → Use `sqlx` with pure Rust driver
- `libz-sys` → Use `flate2` with `miniz_oxide` backend
- `libc` (direct usage) → Use Rust std abstractions
- `bindgen` (generates C bindings) → Not needed for pure Rust project

**Enforcement:**
- `deny.toml` configuration blocks these dependencies
- CI fails if any banned dependency is detected
- Code reviews must verify no C/C++ FFI is introduced

---

## 7. Documentation and Traceability

### 7.1 Rust Version History

| Date | Rust Version | Change Reason | ADR Reference |
|------|--------------|---------------|---------------|
| 2025-10-01 | 1.89.0 | Initial project setup | ADR-RUST-001 |
| <!-- Future --> | <!-- TBD --> | <!-- Quarterly update or security patch --> | <!-- ADR-RUST-002 --> |

### 7.2 Tooling Version Matrix

| Tool | Version | Purpose | C/C++ Free | Verification |
|------|---------|---------|------------|--------------|
| rustc | 1.89.0 | Rust compiler | ✅ Yes | Native Rust tool |
| cargo | 1.89.0 | Build system | ✅ Yes | Native Rust tool |
| clippy | 0.1.89 (included) | Linter | ✅ Yes | Rust component |
| rustfmt | 1.7.0 (included) | Formatter | ✅ Yes | Rust component |
| cargo-llvm-cov | 0.6.19 | Coverage | ✅ Yes | Pure Rust, uses LLVM (no C++ code compiled) |
| cargo-audit | 0.21.2 | Vulnerability scanner | ✅ Yes | Pure Rust |
| cargo-deny | 0.18.4 | Dependency policy | ✅ Yes | Pure Rust |

### 7.3 Developer Onboarding Checklist

**For New Developers:**
- [ ] Install Rust 1.89.0 via rustup
- [ ] Add clippy and rustfmt components
- [ ] Install cargo-llvm-cov, cargo-audit, cargo-deny
- [ ] Configure IDE (VS Code with rust-analyzer)
- [ ] Clone repository and run `cargo build` to verify setup
- [ ] Run `cargo test` to ensure tests pass
- [ ] Run pre-commit hook test: `cargo fmt -- --check && cargo clippy`
- [ ] Review `deny.toml` for banned dependencies
- [ ] Read `CONTRIBUTING.md` for code submission guidelines

---

## 8. Troubleshooting

### 8.1 Common Issues

**Issue: "linker `lld-link.exe` not found" on Windows**
- **Cause:** Missing Visual Studio Build Tools or incorrect Rust toolchain
- **Solution:**
  ```powershell
  # Install Build Tools for Visual Studio 2022
  # OR switch to WSL2 (recommended)
  wsl --install -d Ubuntu-22.04
  ```

**Issue: "cargo-llvm-cov: command not found"**
- **Cause:** Cargo extensions not in PATH
- **Solution:**
  ```bash
  # Ensure ~/.cargo/bin is in PATH
  echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
  source ~/.bashrc
  cargo install cargo-llvm-cov --force
  ```

**Issue: "error: failed to verify the checksum of <crate>"**
- **Cause:** Corrupted cargo cache
- **Solution:**
  ```bash
  rm -rf ~/.cargo/registry/cache
  cargo clean
  cargo build
  ```

### 8.2 Performance Optimization

**Slow Compilation on Windows:**
```powershell
# Enable parallel builds
$env:CARGO_BUILD_JOBS = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors

# Use faster linker (if available)
$env:RUSTFLAGS = "-C link-arg=-fuse-ld=lld"
```

**Faster Incremental Builds:**
```toml
# .cargo/config.toml
[build]
incremental = true
pipelining = true

[profile.dev]
opt-level = 0
debug = true
split-debuginfo = "unpacked"  # Faster on macOS/Linux
```

---

## 9. Continuous Improvement

### 9.1 Quarterly Review Checklist
- [ ] Update Rust to latest stable (test in CI first)
- [ ] Update cargo extensions (`cargo install-update -a`)
- [ ] Review new clippy lints and enable relevant ones
- [ ] Check for deprecated APIs in dependencies
- [ ] Update this document with new version numbers

### 9.2 Security Patch Process
1. Monitor Rust Security Advisory Database (https://rustsec.org/)
2. Apply patches within 48 hours for HIGH/CRITICAL severity
3. Document version change in `decision-log.md` as ADR
4. Notify `#hl-sec-wg` Slack channel after patching

---

## 10. References

- **Official Rust Documentation:** https://doc.rust-lang.org/
- **Cargo Book:** https://doc.rust-lang.org/cargo/
- **Rust Security Advisory Database:** https://rustsec.org/
- **RustCrypto Project:** https://github.com/RustCrypto
- **HoneyLink Architecture:** [spec/architecture/tech-stack.md](../spec/architecture/tech-stack.md)
- **HoneyLink Security:** [spec/security/encryption.md](../spec/security/encryption.md)

---

**Document Control:**
- **Version:** 1.0
- **Last Updated:** 2025-10-01
- **Owner:** ENG-ARCH-01 (Principal Systems Architect)
- **Approval:** Architecture WG
- **Next Review:** 2026-01-01 (Quarterly)
