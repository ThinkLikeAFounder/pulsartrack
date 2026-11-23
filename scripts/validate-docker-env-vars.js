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

function getContractEnvVars(envText) {
  const match = envText.match(/const CONTRACT_ENV_VARS = \[([\s\S]*?)\] as const;/);
  if (!match) return null;
  return match[1]
    .split('\n')
    .map(line => line.trim())
    .filter(line => line.startsWith('"CONTRACT_'))
    .map(line => line.replace(/[",]/g, ''));
}

function getBackendContractEnvVars(composeText) {
  const compose = parseYaml(composeText);
  const environment = compose?.services?.backend?.environment;
  if (!environment) return null;

  const keys = Array.isArray(environment)
    ? environment.map(entry => String(entry).split('=')[0].trim())
    : Object.keys(environment);
  return keys.filter(key => key.startsWith('CONTRACT_'));
}

module.exports = { getFrontendBuildArgs, getContractEnvVars, getBackendContractEnvVars };

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

const backendEnvPath = path.join(__dirname, '../backend/src/config/env.ts');
const backendEnv = fs.readFileSync(backendEnvPath, 'utf8');
const contractEnvVars = getContractEnvVars(backendEnv);
if (!contractEnvVars) {
  console.error('❌ Could not find CONTRACT_ENV_VARS in backend/src/config/env.ts');
  process.exit(1);
}

console.log(`📋 Found ${contractEnvVars.length} contract env vars in CONTRACT_ENV_VARS`);

// Extract build args from the frontend service only
const composeArgs = getFrontendBuildArgs(dockerCompose);
if (!composeArgs) {
  console.error('❌ Could not find frontend build.args in docker-compose.yml');
  process.exit(1);
}

console.log(`📦 Found ${composeArgs.length} NEXT_PUBLIC_* build args in docker-compose.yml`);

const composeContractVars = getBackendContractEnvVars(dockerCompose);
if (!composeContractVars) {
  console.error('❌ Could not find backend environment in docker-compose.yml');
  process.exit(1);
}

console.log(`📦 Found ${composeContractVars.length} CONTRACT_* vars in docker-compose.yml backend environment`);

// Validate: All required vars must be in Dockerfile ARGs
const missingInDockerfile = requiredVars.filter(v => !dockerfileArgs.includes(v));
const missingInCompose = requiredVars.filter(v => !composeArgs.includes(v));
const missingInBackend = contractEnvVars.filter(v => !composeContractVars.includes(v));

// Also check for extras (not required but declared)
const extraInDockerfile = dockerfileArgs.filter(v => !requiredVars.includes(v));
const extraInCompose = composeArgs.filter(v => !requiredVars.includes(v));
const extraInBackend = composeContractVars.filter(v => !contractEnvVars.includes(v));

let hasErrors = false;

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

if (missingInBackend.length > 0) {
  console.error('\n❌ MISSING in docker-compose.yml backend environment:');
  missingInBackend.forEach(v => console.error(`   - ${v}`));
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

if (extraInBackend.length > 0) {
  console.warn('\n⚠️  EXTRA in docker-compose.yml backend environment (not in CONTRACT_ENV_VARS):');
  extraInBackend.forEach(v => console.warn(`   - ${v}`));
}

if (!hasErrors) {
  console.log('\n✅ All required environment variables are declared in both Dockerfile and docker-compose.yml');
  console.log('✅ All backend contract env vars are declared in docker-compose.yml');
  console.log('✅ Docker build configuration is valid');
  process.exit(0);
} else {
  console.error('\n❌ Validation failed: Docker configuration does not match REQUIRED_ENV_VARS or CONTRACT_ENV_VARS');
  console.error('\nTo fix this issue:');
  console.error('1. Add missing ARG declarations to frontend/Dockerfile');
  console.error('2. Add corresponding ENV statements in frontend/Dockerfile');
  console.error('3. Add missing build args to docker-compose.yml frontend service');
  console.error('4. Add missing CONTRACT_* vars to docker-compose.yml backend environment');
  console.error('\nRefer to the existing patterns in both files.');
  process.exit(1);
}
