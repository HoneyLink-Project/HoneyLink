#!/usr/bin/env node

/**
 * Audit for C/C++ native modules in node_modules.
 * Fails CI if any are detected without whitelist approval.
 *
 * Usage: node scripts/audit-native-deps.js
 *
 * Exit codes:
 *   0 - No C/C++ modules detected (success)
 *   1 - C/C++ modules detected or error occurred (failure)
 */

const fs = require('fs');
const path = require('path');

// Whitelist: Pre-approved native modules (requires ADR for each entry)
// Add entries here ONLY after Security WG and Architecture WG approval via ADR
const WHITELIST = [
  // Example: 'some-approved-native-module'
];

// Patterns that indicate C/C++ native module compilation
const BANNED_PATTERNS = [
  'binding.gyp',         // node-gyp build configuration
  'node-gyp',            // node-gyp executable
  'node-pre-gyp',        // Prebuilt binary downloader (often masks native code)
  'prebuild-install',    // Another prebuilt system
  'cmake-js',            // CMake-based build system for Node.js
];

// Suspicious install/postinstall script patterns
const SUSPICIOUS_SCRIPTS = [
  /node-gyp\s+rebuild/i,   // Rebuilding native modules
  /cmake\s+-/i,            // CMake invocation
  /make\s+/i,              // GNU Make invocation
  /g\+\+\s+/i,             // Direct C++ compiler usage
  /clang\s+/i,             // Direct C/C++ compiler usage
  /gcc\s+/i,               // Direct C compiler usage
];

/**
 * Traverse node_modules and detect native modules
 * @param {string} dir - Root directory to start search (typically node_modules)
 * @returns {Array} - List of detected violations
 */
function findNativeModules(dir) {
  const nativeModules = [];

  /**
   * Recursive directory traversal
   * @param {string} currentDir - Current directory being processed
   */
  function traverse(currentDir) {
    let entries;
    try {
      entries = fs.readdirSync(currentDir, { withFileTypes: true });
    } catch (err) {
      console.error(`Warning: Cannot read directory ${currentDir}: ${err.message}`);
      return;
    }

    for (const entry of entries) {
      const fullPath = path.join(currentDir, entry.name);

      if (entry.isDirectory()) {
        // Skip nested node_modules (avoid scanning dependencies of dependencies)
        if (entry.name === 'node_modules' && currentDir !== dir) continue;

        // Check for banned files in package directory
        if (currentDir.includes('node_modules')) {
          const packageName = path.basename(currentDir);

          // Skip if whitelisted
          if (WHITELIST.includes(packageName)) continue;

          for (const pattern of BANNED_PATTERNS) {
            if (entry.name === pattern || entry.name.includes(pattern)) {
              nativeModules.push({
                package: packageName,
                reason: `Found banned file/directory: ${entry.name}`,
                path: fullPath,
              });
              break; // Avoid duplicate reports for same package
            }
          }
        }

        traverse(fullPath);
      } else if (entry.name === 'package.json' && currentDir.includes('node_modules')) {
        // Check package.json for suspicious install scripts
        const packageName = path.basename(currentDir);

        // Skip if whitelisted
        if (WHITELIST.includes(packageName)) continue;

        let packageJson;
        try {
          const content = fs.readFileSync(fullPath, 'utf8');
          packageJson = JSON.parse(content);
        } catch (err) {
          console.error(`Warning: Cannot parse ${fullPath}: ${err.message}`);
          continue;
        }

        const scripts = packageJson.scripts || {};
        for (const [scriptName, scriptContent] of Object.entries(scripts)) {
          if (['install', 'postinstall', 'preinstall'].includes(scriptName)) {
            for (const suspiciousPattern of SUSPICIOUS_SCRIPTS) {
              if (suspiciousPattern.test(scriptContent)) {
                nativeModules.push({
                  package: packageName,
                  reason: `Suspicious ${scriptName} script: "${scriptContent}"`,
                  path: fullPath,
                });
                break; // One violation per script is enough
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

/**
 * Main entry point
 */
function main() {
  const nodeModulesPath = path.join(process.cwd(), 'ui', 'node_modules');

  // Check if node_modules exists
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

    // Group by package for cleaner output
    const grouped = {};
    for (const violation of nativeModules) {
      if (!grouped[violation.package]) {
        grouped[violation.package] = [];
      }
      grouped[violation.package].push(violation);
    }

    for (const [packageName, violations] of Object.entries(grouped)) {
      console.error(`  📦 Package: ${packageName}`);
      for (const v of violations) {
        console.error(`     • ${v.reason}`);
        console.error(`       Path: ${v.path}`);
      }
      console.error('');
    }

    console.error(`Total violations: ${nativeModules.length}\n`);
    console.error('🔧 To resolve:');
    console.error('1. Remove the offending package: pnpm remove <package-name>');
    console.error('2. Find a pure JS/TS/WASM alternative (see docs/NODE_SETUP.md § 5.3)');
    console.error('3. If no alternative exists, submit an ADR for Security WG review');
    console.error('   - ADR must justify the need for C/C++ dependency');
    console.error('   - Security WG must audit the source code and build process');
    console.error('   - Add to WHITELIST in this script after approval\n');

    process.exit(1);
  }
}

// Handle errors gracefully
try {
  main();
} catch (err) {
  console.error('❌ Fatal error during native module audit:');
  console.error(err.message);
  console.error(err.stack);
  process.exit(1);
}
