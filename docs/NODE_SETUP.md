# Node.js + pnpm Development Environment Setup

**Badges:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> This document defines the complete setup procedure for the Node.js + pnpm environment used for HoneyLink™ UI implementation. All dependencies are pure JavaScript/TypeScript/WebAssembly and exclude C/C++ native modules.

---

## 1. System Requirements

### 1.1 Supported Platforms
- **Primary Development:** WSL2 (Ubuntu 22.04 LTS) on Windows 10/11
- **Alternative:** Windows 10/11 native, macOS 13+, Linux (Ubuntu 22.04+)
- **CI/CD:** Linux x86_64 (GitHub Actions runners)

### 1.2 Node.js Version Policy
- **Target Version:** Node.js LTS (Long-Term Support)
- **Current Version:** v22.15.0 (LTS as of 2025-10-01)
- **Version Locking:** Use `.nvmrc` or `package.json#engines` to enforce version
- **Update Cadence:** Migrate to new LTS every 6 months (April/October)

---

## 2. Node.js Installation

### 2.1 Install Node.js LTS

**Option 1: Using NVM (Recommended for macOS/Linux/WSL)**
```bash
# Install NVM (Node Version Manager)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash

# Reload shell configuration
source ~/.bashrc  # or ~/.zshrc for Zsh

# Install Node.js LTS
nvm install --lts

# Verify installation
node --version  # Expected: v22.15.0
npm --version   # Expected: 10.x.x
```

**Option 2: Using Official Installer (Windows/macOS)**
1. Download LTS installer from https://nodejs.org/
2. Run installer and follow prompts
3. Restart terminal after installation
4. Verify installation:
   ```powershell
   node --version  # Expected: v22.15.0
   npm --version   # Expected: 10.x.x
   ```

**Option 3: Using Package Manager (Linux)**
```bash
# Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt-get install -y nodejs

# Verify installation
node --version
npm --version
```

### 2.2 Version Locking

**File:** `.nvmrc` (project root)
```
v22.15.0
```

**File:** `package.json` (project root)
```json
{
  "name": "honeylink-ui",
  "engines": {
    "node": ">=22.15.0 <23.0.0",
    "npm": ">=10.0.0",
    "pnpm": ">=10.0.0"
  }
}
```

---

## 3. pnpm Installation

### 3.1 Install pnpm Globally

**Method 1: Using npm (cross-platform)**
```bash
npm install -g pnpm

# Verify installation
pnpm --version  # Expected: 10.x.x
```

**Method 2: Using standalone script (recommended for CI)**
```bash
# macOS/Linux/WSL
curl -fsSL https://get.pnpm.io/install.sh | sh -

# Windows PowerShell
iwr https://get.pnpm.io/install.ps1 -useb | iex

# Verify installation
pnpm --version
```

**Method 3: Using Corepack (Node.js 16.13+)**
```bash
# Enable Corepack (ships with Node.js)
corepack enable

# Install specific pnpm version
corepack prepare pnpm@10.0.0 --activate

# Verify installation
pnpm --version
```

**Troubleshooting (Windows):**
If `pnpm` command is not found after installation via npm:
1. Close and reopen PowerShell/Terminal
2. Check PATH includes `%APPDATA%\npm` (Windows) or `~/.npm-global/bin` (Linux)
3. Manually add to PATH if needed:
   ```powershell
   $env:Path += ";$env:APPDATA\npm"
   [System.Environment]::SetEnvironmentVariable("Path", $env:Path, "User")
   ```

### 3.2 Why pnpm Over npm/yarn?

| Feature | pnpm | npm | yarn (classic) |
|---------|------|-----|----------------|
| **Disk Efficiency** | ✅ Symlinks to global store | ❌ Duplicates per project | ❌ Duplicates per project |
| **Install Speed** | ✅ Fast (parallel + cache) | ⚠️ Moderate | ✅ Fast |
| **Strict Dependency** | ✅ Prevents phantom deps | ❌ Hoists all deps | ❌ Hoists all deps |
| **C/C++ Detection** | ✅ Easy to audit via scripts | ⚠️ Harder to track | ⚠️ Harder to track |
| **Monorepo Support** | ✅ Native workspace support | ✅ Workspaces (v7+) | ✅ Workspaces |
| **Security** | ✅ Strict by default | ⚠️ Requires `--strict-peer-deps` | ⚠️ Lenient |

**Decision:** pnpm provides strict dependency isolation and efficient disk usage, making it easier to enforce C/C++ dependency exclusion.

---

## 4. TypeScript Installation

### 4.1 Install TypeScript 5.x

**Global Installation (optional, for CLI usage):**
```bash
pnpm add -g typescript

# Verify installation
tsc --version  # Expected: Version 5.x.x
```

**Project-Local Installation (recommended):**
```bash
# Initialize pnpm workspace
pnpm init

# Install TypeScript as dev dependency
pnpm add -D typescript @types/node

# Create tsconfig.json
pnpm tsc --init
```

### 4.2 TypeScript Configuration

**File:** `tsconfig.json` (project root)
```json
{
  "compilerOptions": {
    /* Language and Environment */
    "target": "ES2024",
    "lib": ["ES2024", "DOM", "DOM.Iterable"],
    "jsx": "react-jsx",
    "module": "ESNext",
    "moduleResolution": "bundler",
    
    /* Strict Type Checking */
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "noPropertyAccessFromIndexSignature": true,
    "noFallthroughCasesInSwitch": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "exactOptionalPropertyTypes": true,
    
    /* Interop Constraints */
    "esModuleInterop": true,
    "allowSyntheticDefaultImports": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    
    /* Emit */
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "removeComments": false,
    "importHelpers": true,
    
    /* Advanced */
    "skipLibCheck": true,
    "allowImportingTsExtensions": false,
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*", "tests/**/*"],
  "exclude": ["node_modules", "dist", "build"]
}
```

---

## 5. C/C++ Native Module Exclusion Policy

### 5.1 Policy Statement

**HoneyLink™ C/C++ Exclusion Policy (v1.0)**

All Node.js dependencies MUST be pure JavaScript, TypeScript, or WebAssembly. Dependencies that compile C/C++ native modules during installation are **strictly prohibited** unless explicitly approved by the Architecture and Security Working Groups via ADR.

**Rationale:**
1. **Security:** C/C++ modules introduce memory safety vulnerabilities (buffer overflows, use-after-free)
2. **Supply Chain:** Native modules often depend on system libraries with unknown security posture
3. **Portability:** C/C++ modules require platform-specific compilation, complicating CI/CD
4. **Auditability:** Pure JS/TS/WASM code is easier to audit and analyze statically

### 5.2 Banned Dependency Patterns

**Automatic Rejection (CI Fails):**
- Any package with `binding.gyp` in its repository
- Packages depending on `node-gyp`, `node-pre-gyp`, `prebuild`, `prebuild-install`
- Packages with `install` or `postinstall` scripts that invoke `node-gyp` or `cmake`
- Direct dependencies on native modules: `bcrypt`, `node-canvas`, `sqlite3`, `sharp`, `node-sass`

**Audit Required (Manual Review):**
- Packages with `.node` binary files in `node_modules` after install
- Packages with `optionalDependencies` containing native modules (even if fallback exists)

### 5.3 Pure JavaScript/WASM Alternatives

| Banned Package | Pure Alternative | Reason |
|----------------|------------------|--------|
| `bcrypt` | `@node-rs/bcrypt` (Rust WASM) or `bcryptjs` (pure JS) | Password hashing |
| `node-canvas` | `skia-canvas` (pure JS) or use browser Canvas API | Canvas rendering |
| `sqlite3` | `better-sqlite3` (pure JS binding) or `sql.js` (WASM) | SQLite database |
| `sharp` | `jimp` (pure JS) or `@squoosh/lib` (WASM) | Image processing |
| `node-sass` | `sass` (pure JS Dart Sass) | SASS compilation |
| `node-gyp-build` | `napi-rs` (Rust → WASM) | Native binding alternative |
| `fsevents` (macOS) | `chokidar` with JS fallback | File watching |
| `ws` (C++ accelerated) | `ws` with `--no-optional` flag | WebSocket (disable native bufferutil) |

### 5.4 Allowed Native Modules (Exceptions)

**Pre-Approved by Security WG:**
- **None at project start.** All exceptions require ADR approval.

**Future Exception Process:**
1. Propose dependency via ADR with justification (performance, no pure alternative)
2. Security WG audits source code and build process
3. Architecture WG evaluates portability and maintenance burden
4. If approved, add to whitelist in `.pnpmrc` and document in ADR

---

## 6. Dependency Installation and Management

### 6.1 Initialize UI Project

**Create UI Workspace:**
```bash
# Navigate to project root
cd HoneyLink

# Create ui directory
mkdir -p ui
cd ui

# Initialize pnpm project
pnpm init

# Install Vite + React + TypeScript template
pnpm create vite . --template react-ts

# Install dependencies
pnpm install

# Verify no C/C++ modules
pnpm list --depth=Infinity | grep -E "(binding.gyp|node-gyp|prebuild)"
# Expected: No output (empty result)
```

### 6.2 Configure pnpm Workspace

**File:** `pnpm-workspace.yaml` (project root)
```yaml
packages:
  - 'ui'
  - 'backend/crates/*'  # Rust crates (not managed by pnpm, but declared for completeness)
```

**File:** `.pnpmrc` (project root)
```
# Enforce strict peer dependencies
strict-peer-dependencies=true

# Hoist only essential packages (minimize risk of phantom deps)
public-hoist-pattern[]=*eslint*
public-hoist-pattern[]=*prettier*

# Block unsafe packages (abort install if detected)
# Note: pnpm does not have native package blocking; use custom script

# Auto-install peers (avoid manual peer dep management)
auto-install-peers=true

# Use hard links for performance (optional, disable on Windows if issues)
# node-linker=hoisted  # Uncomment if symlink issues arise

# Registry mirror (optional, for corporate proxies)
# registry=https://registry.npmjs.org/
```

### 6.3 Dependency Audit Script

**File:** `scripts/audit-native-deps.js` (project root)
```javascript
#!/usr/bin/env node

/**
 * Audit for C/C++ native modules in node_modules.
 * Fails CI if any are detected without whitelist approval.
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const WHITELIST = [
  // Add approved native modules here (requires ADR)
  // Example: 'some-approved-native-module'
];

const BANNED_PATTERNS = [
  'binding.gyp',
  'node-gyp',
  'node-pre-gyp',
  'prebuild-install',
  'cmake-js',
];

const SUSPICIOUS_SCRIPTS = [
  /node-gyp\s+rebuild/,
  /cmake\s+-/,
  /make\s+/,
  /g\+\+\s+/,
];

function findNativeModules(dir) {
  const nativeModules = [];
  
  function traverse(currentDir) {
    const entries = fs.readdirSync(currentDir, { withFileTypes: true });
    
    for (const entry of entries) {
      const fullPath = path.join(currentDir, entry.name);
      
      if (entry.isDirectory()) {
        // Skip nested node_modules
        if (entry.name === 'node_modules' && currentDir !== dir) continue;
        
        // Check for banned files in package directory
        if (currentDir.includes('node_modules')) {
          const packageName = path.basename(currentDir);
          
          for (const pattern of BANNED_PATTERNS) {
            if (entry.name === pattern || entry.name.includes(pattern)) {
              if (!WHITELIST.includes(packageName)) {
                nativeModules.push({
                  package: packageName,
                  reason: `Found banned file: ${entry.name}`,
                  path: fullPath,
                });
              }
            }
          }
        }
        
        traverse(fullPath);
      } else if (entry.name === 'package.json' && currentDir.includes('node_modules')) {
        // Check package.json for suspicious install scripts
        const packageJson = JSON.parse(fs.readFileSync(fullPath, 'utf8'));
        const packageName = path.basename(currentDir);
        
        if (WHITELIST.includes(packageName)) continue;
        
        const scripts = packageJson.scripts || {};
        for (const [scriptName, scriptContent] of Object.entries(scripts)) {
          if (['install', 'postinstall', 'preinstall'].includes(scriptName)) {
            for (const suspiciousPattern of SUSPICIOUS_SCRIPTS) {
              if (suspiciousPattern.test(scriptContent)) {
                nativeModules.push({
                  package: packageName,
                  reason: `Suspicious ${scriptName} script: ${scriptContent}`,
                  path: fullPath,
                });
              }
            }
          }
        }
      }
    }
  }
  
  traverse(dir);
  return nativeModules;
}

function main() {
  const nodeModulesPath = path.join(process.cwd(), 'ui', 'node_modules');
  
  if (!fs.existsSync(nodeModulesPath)) {
    console.log('✅ No node_modules found. Run `pnpm install` first.');
    process.exit(0);
  }
  
  console.log('🔍 Auditing for C/C++ native modules...\n');
  
  const nativeModules = findNativeModules(nodeModulesPath);
  
  if (nativeModules.length === 0) {
    console.log('✅ No C/C++ native modules detected. All dependencies are pure JS/TS/WASM.\n');
    process.exit(0);
  } else {
    console.error('❌ C/C++ native modules detected:\n');
    nativeModules.forEach(({ package, reason, path }) => {
      console.error(`  Package: ${package}`);
      console.error(`  Reason:  ${reason}`);
      console.error(`  Path:    ${path}\n`);
    });
    console.error(`Total violations: ${nativeModules.length}`);
    console.error('\nTo resolve:');
    console.error('1. Remove the offending package: pnpm remove <package-name>');
    console.error('2. Find a pure JS/TS/WASM alternative (see docs/NODE_SETUP.md § 5.3)');
    console.error('3. If no alternative exists, submit an ADR for Security WG review\n');
    process.exit(1);
  }
}

main();
```

**Usage:**
```bash
# Make script executable
chmod +x scripts/audit-native-deps.js

# Run manually
node scripts/audit-native-deps.js

# Integrate into package.json
# Add to ui/package.json:
{
  "scripts": {
    "postinstall": "node ../scripts/audit-native-deps.js"
  }
}
```

---

## 7. Essential Dependencies

### 7.1 UI Framework Stack (No C/C++ Dependencies)

**File:** `ui/package.json`
```json
{
  "name": "honeylink-ui",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "engines": {
    "node": ">=22.15.0 <23.0.0",
    "pnpm": ">=10.0.0"
  },
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "lint": "eslint . --ext .ts,.tsx",
    "lint:fix": "eslint . --ext .ts,.tsx --fix",
    "format": "prettier --write \"src/**/*.{ts,tsx,css}\"",
    "type-check": "tsc --noEmit",
    "postinstall": "node ../scripts/audit-native-deps.js"
  },
  "dependencies": {
    "react": "^18.3.1",
    "react-dom": "^18.3.1",
    "react-router-dom": "^7.1.3",
    "@tanstack/react-query": "^6.0.0",
    "zustand": "^5.0.2",
    "clsx": "^2.1.1",
    "date-fns": "^4.1.0"
  },
  "devDependencies": {
    "@types/react": "^18.3.18",
    "@types/react-dom": "^18.3.5",
    "@types/node": "^22.10.5",
    "@vitejs/plugin-react": "^4.3.4",
    "typescript": "^5.7.2",
    "vite": "^6.0.7",
    "eslint": "^9.20.0",
    "eslint-plugin-react": "^7.37.3",
    "eslint-plugin-react-hooks": "^5.1.0",
    "@typescript-eslint/eslint-plugin": "^8.20.0",
    "@typescript-eslint/parser": "^8.20.0",
    "prettier": "^3.4.2",
    "tailwindcss": "^3.4.17",
    "postcss": "^8.4.49",
    "autoprefixer": "^10.4.21"
  }
}
```

### 7.2 Verified Pure JS/TS Packages

**Core Dependencies:**
- ✅ `react` / `react-dom`: Pure JS library
- ✅ `react-router-dom`: Pure JS routing
- ✅ `@tanstack/react-query`: Pure TS data fetching
- ✅ `zustand`: Pure TS state management (no Redux C++ bindings)
- ✅ `clsx`: Pure JS className utility
- ✅ `date-fns`: Pure JS date library (alternative to moment.js)

**Build Tools:**
- ✅ `vite`: Pure JS bundler (uses esbuild in WASM mode)
- ✅ `typescript`: Pure TS compiler
- ✅ `eslint`: Pure JS linter
- ✅ `prettier`: Pure JS formatter
- ✅ `tailwindcss`: Pure JS CSS framework
- ✅ `postcss`: Pure JS CSS processor

**Testing (to be added):**
- ✅ `vitest`: Pure JS test runner (Vite-native)
- ✅ `@testing-library/react`: Pure JS testing utilities
- ✅ `playwright`: Browser automation (pure JS, downloads browsers)

**Avoid (C/C++ Dependencies):**
- ❌ `webpack`: Often pulls in native dependencies (use Vite instead)
- ❌ `node-sass`: C++ libsass (use `sass` / Dart Sass instead)
- ❌ `karma`: Older test runner with native deps (use Vitest/Playwright instead)

---

## 8. CI/CD Integration

### 8.1 GitHub Actions Workflow

**File:** `.github/workflows/ui-ci.yml`
```yaml
name: UI CI

on:
  push:
    branches: [master, develop]
    paths:
      - 'ui/**'
      - 'scripts/audit-native-deps.js'
  pull_request:
    branches: [master, develop]
    paths:
      - 'ui/**'

env:
  NODE_VERSION: '22.15.0'
  PNPM_VERSION: '10.0.0'

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - uses: pnpm/action-setup@v4
        with:
          version: ${{ env.PNPM_VERSION }}
      
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: 'pnpm'
          cache-dependency-path: 'ui/pnpm-lock.yaml'
      
      - name: Install dependencies
        run: |
          cd ui
          pnpm install --frozen-lockfile
      
      - name: Audit for C/C++ native modules
        run: node scripts/audit-native-deps.js
      
      - name: ESLint
        run: |
          cd ui
          pnpm lint
      
      - name: Prettier check
        run: |
          cd ui
          pnpm exec prettier --check "src/**/*.{ts,tsx,css}"
  
  type-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - uses: pnpm/action-setup@v4
        with:
          version: ${{ env.PNPM_VERSION }}
      
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: 'pnpm'
          cache-dependency-path: 'ui/pnpm-lock.yaml'
      
      - name: Install dependencies
        run: |
          cd ui
          pnpm install --frozen-lockfile
      
      - name: TypeScript type check
        run: |
          cd ui
          pnpm type-check
  
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - uses: pnpm/action-setup@v4
        with:
          version: ${{ env.PNPM_VERSION }}
      
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: 'pnpm'
          cache-dependency-path: 'ui/pnpm-lock.yaml'
      
      - name: Install dependencies
        run: |
          cd ui
          pnpm install --frozen-lockfile
      
      - name: Build production bundle
        run: |
          cd ui
          pnpm build
      
      - name: Upload build artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ui-build
          path: ui/dist/
          retention-days: 7
```

---

## 9. Developer Workflow

### 9.1 Daily Development Commands

```bash
# Start development server with HMR
cd ui
pnpm dev
# Open http://localhost:5173

# Run type checking in watch mode (separate terminal)
pnpm type-check --watch

# Run linter
pnpm lint

# Auto-fix linting issues
pnpm lint:fix

# Format all files
pnpm format

# Build for production
pnpm build

# Preview production build
pnpm preview
```

### 9.2 Pre-Commit Checklist

**Automated (via Git hooks or manual):**
```bash
# Format check
pnpm exec prettier --check "src/**/*.{ts,tsx,css}"

# Lint check
pnpm lint

# Type check
pnpm type-check

# Audit native modules
node ../scripts/audit-native-deps.js
```

**Manual Review:**
- No `console.log` statements in committed code (use proper logging library)
- All new components have basic tests
- README.md updated if public API changes

---

## 10. Documentation and Traceability

### 10.1 Node.js Version History

| Date | Node.js Version | pnpm Version | Change Reason | ADR Reference |
|------|-----------------|--------------|---------------|---------------|
| 2025-10-01 | 22.15.0 | 10.0.0 | Initial project setup | ADR-NODE-001 |
| <!-- Future --> | <!-- TBD --> | <!-- TBD --> | <!-- LTS migration --> | <!-- ADR-NODE-002 --> |

### 10.2 Dependency Audit Log

| Date | Package | Version | Action | Reason |
|------|---------|---------|--------|--------|
| 2025-10-01 | All dependencies | Initial | Approved | Pure JS/TS/WASM verified |
| <!-- Future --> | <!-- TBD --> | <!-- TBD --> | <!-- Added/Removed --> | <!-- Rationale --> |

### 10.3 Developer Onboarding Checklist

**For New UI Developers:**
- [ ] Install Node.js 22.15.0 (LTS) via nvm or official installer
- [ ] Install pnpm globally: `npm install -g pnpm`
- [ ] Clone repository and navigate to `ui/` directory
- [ ] Run `pnpm install` to install dependencies
- [ ] Run `node ../scripts/audit-native-deps.js` to verify no C/C++ modules
- [ ] Run `pnpm dev` to start development server
- [ ] Configure IDE (VS Code with ESLint/Prettier extensions)
- [ ] Review `ui/README.md` for project-specific conventions
- [ ] Read C/C++ Exclusion Policy (this document § 5)

---

## 11. Troubleshooting

### 11.1 Common Issues

**Issue: `pnpm: command not found` after installation**
- **Cause:** PATH not updated or terminal not restarted
- **Solution:**
  ```powershell
  # Windows PowerShell
  $env:Path += ";$env:APPDATA\npm"
  # Restart terminal after adding to PATH
  ```

**Issue: `Error: Cannot find module 'X'`**
- **Cause:** Stale lock file or incomplete install
- **Solution:**
  ```bash
  rm -rf node_modules pnpm-lock.yaml
  pnpm install
  ```

**Issue: `ELIFECYCLE Command failed` during `pnpm install`**
- **Cause:** Native module trying to compile
- **Solution:**
  ```bash
  # Identify the failing package from error output
  pnpm list --depth=Infinity | grep <failing-package>
  # Remove the package and find pure JS/TS alternative
  pnpm remove <failing-package>
  ```

**Issue: Vite HMR not working on Windows**
- **Cause:** File watcher limitations on Windows
- **Solution:**
  ```javascript
  // vite.config.ts
  export default defineConfig({
    server: {
      watch: {
        usePolling: true,  // Enable polling for Windows
      },
    },
  });
  ```

### 11.2 Performance Optimization

**Speed up pnpm install:**
```bash
# Use offline mode if dependencies are cached
pnpm install --offline

# Skip optional dependencies
pnpm install --no-optional

# Use shamefully-hoist for problematic packages (last resort)
# Add to .pnpmrc:
# shamefully-hoist=true
```

---

## 12. References

- **Node.js Official Docs:** https://nodejs.org/docs/
- **pnpm Documentation:** https://pnpm.io/
- **TypeScript Handbook:** https://www.typescriptlang.org/docs/
- **Vite Guide:** https://vite.dev/guide/
- **React Documentation:** https://react.dev/
- **HoneyLink UI Specs:** [spec/ui/overview.md](../spec/ui/overview.md)
- **C/C++ Exclusion ADR:** [spec/notes/decision-log.md](../spec/notes/decision-log.md) (ADR-NODE-001)

---

**Document Control:**
- **Version:** 1.0
- **Last Updated:** 2025-10-01
- **Owner:** UX-LEAD-01 (Experience Director)
- **Approval:** UX WG + Architecture WG
- **Next Review:** 2026-04-01 (LTS migration checkpoint)
