#!/usr/bin/env node

/**
 * Sync version from package.json to tauri.conf.json
 * Run this script before building to ensure versions are in sync
 */

import { readFileSync, writeFileSync } from 'fs'
import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const rootDir = resolve(__dirname, '..')

// Read package.json version
const packageJsonPath = resolve(rootDir, 'package.json')
const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf-8'))
const version = packageJson.version

// Update tauri.conf.json
const tauriConfigPath = resolve(rootDir, 'src-tauri/tauri.conf.json')
const tauriConfig = JSON.parse(readFileSync(tauriConfigPath, 'utf-8'))

if (tauriConfig.version !== version) {
  tauriConfig.version = version
  writeFileSync(tauriConfigPath, JSON.stringify(tauriConfig, null, 2) + '\n')
  console.log(`✓ Synced version to ${version} in tauri.conf.json`)
} else {
  console.log(`✓ Version already in sync: ${version}`)
}
