import { describe, expect, it } from 'vitest';
// eslint-disable-next-line @typescript-eslint/no-require-imports
const {
  getBackendContractEnvVars,
  getBackendEnvExampleContractVars,
  getContractEnvVarMismatch,
  getFrontendBuildArgs,
} = require('../../../scripts/validate-docker-env-vars.js');

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

describe('backend contract environment validation', () => {
  const envSource = `const CONTRACT_ENV_VARS = [
  "CONTRACT_ACCESS_CONTROL",
  "CONTRACT_AD_REGISTRY",
] as const;`;

  it('extracts backend contract variables and example entries', () => {
    const example = 'CONTRACT_ACCESS_CONTROL=\nCONTRACT_AD_REGISTRY=\n';
    expect(getBackendContractEnvVars(envSource)).toEqual([
      'CONTRACT_ACCESS_CONTROL',
      'CONTRACT_AD_REGISTRY',
    ]);
    expect(getBackendEnvExampleContractVars(example)).toEqual([
      'CONTRACT_ACCESS_CONTROL',
      'CONTRACT_AD_REGISTRY',
    ]);
  });

  it('reports missing and fictional contract variables', () => {
    expect(getContractEnvVarMismatch(
      envSource,
      'CONTRACT_ACCESS_CONTROL=\nCONTRACT_CAMPAIGN_SCHEDULER=\n',
    )).toEqual({
      missing: ['CONTRACT_AD_REGISTRY'],
      extra: ['CONTRACT_CAMPAIGN_SCHEDULER'],
      duplicates: [],
      nonEmpty: [],
    });
  });

  it('reports duplicate contract entries and non-blank example values', () => {
    expect(getContractEnvVarMismatch(
      envSource,
      'CONTRACT_ACCESS_CONTROL=filled-in\nCONTRACT_ACCESS_CONTROL=\nCONTRACT_AD_REGISTRY=\n',
    )).toEqual({
      missing: [],
      extra: [],
      duplicates: ['CONTRACT_ACCESS_CONTROL'],
      nonEmpty: ['CONTRACT_ACCESS_CONTROL'],
    });
  });

  it('returns null when the backend contract list cannot be found', () => {
    expect(getBackendContractEnvVars('const OTHER_VARS = [];')).toBeNull();
  });
});
