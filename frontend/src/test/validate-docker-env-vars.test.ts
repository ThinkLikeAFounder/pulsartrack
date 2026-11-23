import { describe, expect, it } from 'vitest';
// eslint-disable-next-line @typescript-eslint/no-require-imports
const validateScript = require('../../../scripts/validate-docker-env-vars.js');
const { getFrontendBuildArgs, getContractEnvVars, getBackendContractEnvVars } = validateScript;

const compose = `
services:
  backend:
    build:
      args:
        NEXT_PUBLIC_NOT_FRONTEND: x
    container_name: backend
  frontend:
    build:
      context: ./frontend
      args:
        NEXT_PUBLIC_API_URL: http://localhost:3001
        NEXT_PUBLIC_SENTRY_DSN: \${NEXT_PUBLIC_SENTRY_DSN:-}
        NEXT_PUBLIC_CONTRACT_AD_REGISTRY: \${CONTRACT_AD_REGISTRY:-PLACEHOLDER}
    container_name: frontend
`;

describe('getFrontendBuildArgs', () => {
  it('returns each frontend build arg once, ignoring names inside values', () => {
    expect(getFrontendBuildArgs(compose)).toEqual([
      'NEXT_PUBLIC_API_URL',
      'NEXT_PUBLIC_SENTRY_DSN',
      'NEXT_PUBLIC_CONTRACT_AD_REGISTRY',
    ]);
  });

  it('supports the list form of build args', () => {
    const listForm = `
services:
  frontend:
    build:
      args:
        - NEXT_PUBLIC_API_URL=http://localhost:3001
        - NEXT_PUBLIC_SENTRY_DSN
`;
    expect(getFrontendBuildArgs(listForm)).toEqual([
      'NEXT_PUBLIC_API_URL',
      'NEXT_PUBLIC_SENTRY_DSN',
    ]);
  });

  it('returns null when the frontend service has no build args', () => {
    expect(getFrontendBuildArgs('services:\n  frontend:\n    image: x\n')).toBeNull();
  });
});

describe('getContractEnvVars', () => {
  it('extracts CONTRACT_* names from CONTRACT_ENV_VARS', () => {
    const envTs = `
export const FOO = ['BAR'] as const;
const CONTRACT_ENV_VARS = [
  "CONTRACT_ACCESS_CONTROL",
  "CONTRACT_AD_REGISTRY",
] as const;
`;
    expect(getContractEnvVars(envTs)).toEqual([
      'CONTRACT_ACCESS_CONTROL',
      'CONTRACT_AD_REGISTRY',
    ]);
  });

  it('returns null when CONTRACT_ENV_VARS is absent', () => {
    expect(getContractEnvVars('export const FOO = 1;')).toBeNull();
  });
});

describe('getBackendContractEnvVars', () => {
  it('returns only CONTRACT_* keys from the backend environment', () => {
    const backendCompose = `
services:
  backend:
    environment:
      NODE_ENV: development
      CONTRACT_ESCROW_VAULT: \${CONTRACT_ESCROW_VAULT:-PLACEHOLDER}
      CONTRACT_AD_REGISTRY: \${CONTRACT_AD_REGISTRY:-PLACEHOLDER}
  frontend:
    environment:
      CONTRACT_IGNORED: x
`;
    expect(getBackendContractEnvVars(backendCompose)).toEqual([
      'CONTRACT_ESCROW_VAULT',
      'CONTRACT_AD_REGISTRY',
    ]);
  });

  it('supports the list form of environment entries', () => {
    const listForm = `
services:
  backend:
    environment:
      - NODE_ENV=development
      - CONTRACT_AD_REGISTRY=\${CONTRACT_AD_REGISTRY:-PLACEHOLDER}
`;
    expect(getBackendContractEnvVars(listForm)).toEqual(['CONTRACT_AD_REGISTRY']);
  });

  it('returns null when the backend service has no environment', () => {
    expect(getBackendContractEnvVars('services:\n  backend:\n    image: x\n')).toBeNull();
  });
});
