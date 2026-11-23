#!/usr/bin/env node
/**
 * Validate Docker Environment Variables
 * Issue #996: Ensures Dockerfile ARGs and docker-compose build.args match REQUIRED_ENV_VARS
 *
 * This script prevents silent failures where contract addresses are missing from
 * the Docker build configuration, which would result in empty strings being baked
 * into the production bundle.
 */

const fs = require('fs');
const path = require('path');
const { createRequire } = require('module');

// The YAML parser is a frontend devDependency; resolve it from there so this
// script needs no package.json of its own.
const { parse: parseYaml } = createRequire(path.join(__dirname, '../frontend/package.json'))('yaml');

/**
 * Returns the NEXT_PUBLIC_* keys declared under services.frontend.build.args.
 * Accepts both the map form (KEY: value) and the list form (- KEY=value).
 */
function getFrontendBuildArgs(composeText) {
  const compose = parseYaml(composeText);
  const args = compose?.services?.frontend?.build?.args;
  if (!args) return null;

  const keys = Array.isArray(args)
    ? args.map(entry => String(entry).split('=')[0].trim())
    : Object.keys(args);
  return keys.filter(key => key.startsWith('NEXT_PUBLIC_'));
}

function getBackendContractEnvVars(envSource) {
  const varsMatch = envSource.match(/const CONTRACT_ENV_VARS = \[([\s\S]*?)\] as const;/);
  if (!varsMatch) return null;

  return [...varsMatch[1].matchAll(/"(CONTRACT_[A-Z0-9_]+)"/g)].map(match => match[1]);
}

function getBackendEnvExampleContractVars(exampleText) {
  return [...exampleText.matchAll(/^[ \t]*(CONTRACT_[A-Z0-9_]+)=/gm)]
    .map(match => match[1]);
}

function getContractEnvVarMismatch(envSource, exampleText) {
  const requiredVars = getBackendContractEnvVars(envSource);
  if (!requiredVars) return null;

  const exampleEntries = [...exampleText.matchAll(/^[ \t]*(CONTRACT_[A-Z0-9_]+)=(.*)$/gm)];
  const exampleVars = exampleEntries.map(match => match[1]);
  const seen = new Set();
  const duplicates = exampleVars.filter(name => {
    if (seen.has(name)) return true;
    seen.add(name);
    return false;
  });
  return {
    missing: requiredVars.filter(name => !exampleVars.includes(name)),
    extra: exampleVars.filter(name => !requiredVars.includes(name)),
    duplicates,
    nonEmpty: exampleEntries
      .filter(match => match[2].trim() !== '')
      .map(match => match[1]),
  };
}

module.exports = {
  getBackendContractEnvVars,
  getBackendEnvExampleContractVars,
  getContractEnvVarMismatch,
  getFrontendBuildArgs,
};

if (require.main !== module) return;

// Read the required env vars from stellar-config.ts
const stellarConfigPath = path.join(__dirname, '../frontend/src/lib/stellar-config.ts');
const stellarConfig = fs.readFileSync(stellarConfigPath, 'utf8');

// Extract REQUIRED_ENV_VARS array
const requiredVarsMatch = stellarConfig.match(/export const REQUIRED_ENV_VARS = \[([\s\S]*?)\] as const;/);
if (!requiredVarsMatch) {
  console.error('❌ Could not find REQUIRED_ENV_VARS in stellar-config.ts');
  process.exit(1);
}

const requiredVars = requiredVarsMatch[1]
  .split('\n')
  .map(line => line.trim())
  .filter(line => line.startsWith("'NEXT_PUBLIC_"))
  .map(line => line.replace(/[',]/g, ''));

console.log(`📋 Found ${requiredVars.length} required environment variables in REQUIRED_ENV_VARS`);

// Read Dockerfile
const dockerfilePath = path.join(__dirname, '../frontend/Dockerfile');
const dockerfile = fs.readFileSync(dockerfilePath, 'utf8');

// Extract ARG declarations
const argPattern = /^ARG (NEXT_PUBLIC_[A-Z_]+)$/gm;
const dockerfileArgs = [];
let match;
while ((match = argPattern.exec(dockerfile)) !== null) {
  dockerfileArgs.push(match[1]);
}

console.log(`🐳 Found ${dockerfileArgs.length} NEXT_PUBLIC_* ARG declarations in Dockerfile`);

// Read docker-compose.yml
const dockerComposePath = path.join(__dirname, '../docker-compose.yml');
const dockerCompose = fs.readFileSync(dockerComposePath, 'utf8');
let hasErrors = false;

// Ensure backend/.env.example stays aligned with the backend's validated variables.
const backendEnvPath = path.join(__dirname, '../backend/src/config/env.ts');
const backendEnv = fs.readFileSync(backendEnvPath, 'utf8');
const backendExamplePath = path.join(__dirname, '../backend/.env.example');
const backendExample = fs.readFileSync(backendExamplePath, 'utf8');
const contractVarMismatch = getContractEnvVarMismatch(backendEnv, backendExample);
if (!contractVarMismatch) {
  console.error('❌ Could not find CONTRACT_ENV_VARS in backend/src/config/env.ts');
  process.exit(1);
}
console.log(`📋 Found ${getBackendContractEnvVars(backendEnv).length} backend contract variables`);

if (contractVarMismatch.missing.length > 0) {
  console.error('\n❌ MISSING in backend/.env.example:');
  contractVarMismatch.missing.forEach(v => console.error(`   - ${v}`));
  hasErrors = true;
}

if (contractVarMismatch.extra.length > 0) {
  console.error('\n❌ EXTRA in backend/.env.example (not in CONTRACT_ENV_VARS):');
  contractVarMismatch.extra.forEach(v => console.error(`   - ${v}`));
  hasErrors = true;
}

if (contractVarMismatch.duplicates.length > 0) {
  console.error('\n❌ DUPLICATED in backend/.env.example:');
  contractVarMismatch.duplicates.forEach(v => console.error(`   - ${v}`));
  hasErrors = true;
}

if (contractVarMismatch.nonEmpty.length > 0) {
  console.error('\n❌ Contract values in backend/.env.example must be blank:');
  contractVarMismatch.nonEmpty.forEach(v => console.error(`   - ${v}`));
  hasErrors = true;
}

// Extract build args from the frontend service only
const composeArgs = getFrontendBuildArgs(dockerCompose);
if (!composeArgs) {
  console.error('❌ Could not find frontend build.args in docker-compose.yml');
  process.exit(1);
}

console.log(`📦 Found ${composeArgs.length} NEXT_PUBLIC_* build args in docker-compose.yml`);

// Validate: All required vars must be in Dockerfile ARGs
const missingInDockerfile = requiredVars.filter(v => !dockerfileArgs.includes(v));
const missingInCompose = requiredVars.filter(v => !composeArgs.includes(v));

// Also check for extras (not required but declared)
const extraInDockerfile = dockerfileArgs.filter(v => !requiredVars.includes(v));
const extraInCompose = composeArgs.filter(v => !requiredVars.includes(v));

if (missingInDockerfile.length > 0) {
  console.error('\n❌ MISSING in Dockerfile ARGs:');
  missingInDockerfile.forEach(v => console.error(`   - ${v}`));
  hasErrors = true;
}

if (missingInCompose.length > 0) {
  console.error('\n❌ MISSING in docker-compose.yml build.args:');
  missingInCompose.forEach(v => console.error(`   - ${v}`));
  hasErrors = true;
}

if (extraInDockerfile.length > 0) {
  console.warn('\n⚠️  EXTRA in Dockerfile ARGs (not in REQUIRED_ENV_VARS):');
  extraInDockerfile.forEach(v => console.warn(`   - ${v}`));
  // Note: This is just a warning, not an error
}

if (extraInCompose.length > 0) {
  console.warn('\n⚠️  EXTRA in docker-compose.yml build.args (not in REQUIRED_ENV_VARS):');
  extraInCompose.forEach(v => console.warn(`   - ${v}`));
  // Note: This is just a warning, not an error
}

if (!hasErrors) {
  console.log('\n✅ Backend contract example and frontend Docker build configuration are valid');
  process.exit(0);
} else {
  console.error('\n❌ Validation failed: environment examples do not match backend and frontend requirements');
  console.error('\nTo fix this issue:');
  console.error('1. Align backend/.env.example contract entries with CONTRACT_ENV_VARS');
  console.error('2. Add missing ARG declarations to frontend/Dockerfile');
  console.error('3. Add corresponding ENV statements and docker-compose.yml build args');
  console.error('\nRefer to the existing patterns in both files.');
  process.exit(1);
}
