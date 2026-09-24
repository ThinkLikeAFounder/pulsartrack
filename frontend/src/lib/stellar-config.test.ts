import { describe, it, expect } from 'vitest';
import {
  xlmToStroops,
  stroopsToXlm,
  getExplorerTxUrl,
  getExplorerAddressUrl,
  getExplorerContractUrl,
  getHorizonUrl,
  getSorobanRpcUrl,
  getNetworkPassphrase,
  validateRequiredEnv,
  REQUIRED_ENV_VARS,
  NETWORKS,
  STROOPS_PER_XLM,
} from './stellar-config';

describe('xlmToStroops', () => {
  it('converts whole XLM amounts exactly', () => {
    expect(xlmToStroops(1)).toBe(BigInt(10_000_000));
    expect(xlmToStroops(0)).toBe(BigInt(0));
  });

  it('converts decimal amounts without floating-point error', () => {
    expect(xlmToStroops(0.57)).toBe(BigInt(5_700_000));
    expect(xlmToStroops(19.99)).toBe(BigInt(199_900_000));
  });

  it('converts one stroop exactly', () => {
    expect(xlmToStroops(0.0000001)).toBe(BigInt(1));
  });

  it('converts large amounts exactly', () => {
    expect(xlmToStroops(1_000_000)).toBe(BigInt(10_000_000_000_000));
  });

  it('throws on non-finite values', () => {
    expect(() => xlmToStroops(NaN)).toThrow('not a finite number');
    expect(() => xlmToStroops(Infinity)).toThrow('not a finite number');
    expect(() => xlmToStroops(-Infinity)).toThrow('not a finite number');
  });

  it('throws on negative values', () => {
    expect(() => xlmToStroops(-1)).toThrow('cannot be negative');
  });

  it('throws on amounts with more than 7 decimal places', () => {
    expect(() => xlmToStroops(0.12345678)).toThrow('more than 7 decimal places');
  });
});

describe('stroopsToXlm', () => {
  it('converts stroops to XLM string without precision loss', () => {
    expect(stroopsToXlm(10_000_000n)).toBe('1');
    expect(stroopsToXlm(0n)).toBe('0');
    expect(stroopsToXlm(1n)).toBe('0.0000001');
  });

  it('converts decimal stroop amounts correctly', () => {
    expect(stroopsToXlm(5_700_000n)).toBe('0.57');
    expect(stroopsToXlm(199_900_000n)).toBe('19.99');
  });

  it('handles number inputs', () => {
    expect(stroopsToXlm(10_000_000)).toBe('1');
  });

  it('trims trailing zeros', () => {
    expect(stroopsToXlm(5_000_000n)).toBe('0.5');
    expect(stroopsToXlm(5_100_000n)).toBe('0.51');
  });
});

describe('round-trip conversions', () => {
  it('xlmToStroops -> stroopsToXlm preserves value', () => {
    const amounts = [0, 0.0000001, 0.57, 1, 19.99, 100, 1_000_000];
    for (const xlm of amounts) {
      const stroops = xlmToStroops(xlm);
      const back = stroopsToXlm(stroops);
      expect(back).toBe(xlm.toString());
    }
  });
});

describe('Explorer URLs', () => {
  it('generates correct mainnet transaction URL', () => {
    process.env.NEXT_PUBLIC_NETWORK = 'mainnet';
    const url = getExplorerTxUrl('abc123');
    expect(url).toContain('stellar.expert');
    expect(url).toContain('public');
    expect(url).toContain('abc123');
  });

  it('generates correct testnet transaction URL', () => {
    process.env.NEXT_PUBLIC_NETWORK = 'testnet';
    const url = getExplorerTxUrl('abc123');
    expect(url).toContain('stellar.expert');
    expect(url).toContain('testnet');
    expect(url).toContain('abc123');
  });

  it('generates address URLs', () => {
    const address = 'GABC123';
    const url = getExplorerAddressUrl(address);
    expect(url).toContain('stellar.expert');
    expect(url).toContain('account');
    expect(url).toContain(address);
  });

  it('generates contract URLs', () => {
    const contractId = 'CABC123';
    const url = getExplorerContractUrl(contractId);
    expect(url).toContain('stellar.expert');
    expect(url).toContain('contract');
    expect(url).toContain(contractId);
  });
});

describe('Network configuration URLs', () => {
  it('returns Horizon URLs for networks', () => {
    const horizonUrl = getHorizonUrl();
    expect(horizonUrl).toContain('horizon');
  });

  it('returns Soroban RPC URLs for networks', () => {
    const sorobanUrl = getSorobanRpcUrl();
    expect(sorobanUrl).toContain('soroban');
  });

  it('returns network passphrase', () => {
    const passphrase = getNetworkPassphrase();
    expect(passphrase).toMatch(/Stellar Network/);
  });
});

describe('validateRequiredEnv', () => {
  it('does not throw when all required env vars are set', () => {
    process.env.NEXT_PUBLIC_NETWORK = 'testnet';
    process.env.NEXT_PUBLIC_WS_URL = 'ws://localhost';
    process.env.NEXT_PUBLIC_CONTRACT_AD_REGISTRY = 'CAD_REGISTRY';
    process.env.NEXT_PUBLIC_CONTRACT_ANALYTICS_AGGREGATOR = 'CANALYTICS';
    process.env.NEXT_PUBLIC_CONTRACT_AUCTION_ENGINE = 'CAUCTION';
    process.env.NEXT_PUBLIC_CONTRACT_CAMPAIGN_ORCHESTRATOR = 'CCAMPAIGN';
    process.env.NEXT_PUBLIC_CONTRACT_DISPUTE_RESOLUTION = 'CDISPUTE';
    process.env.NEXT_PUBLIC_CONTRACT_ESCROW_VAULT = 'CESCROW';
    process.env.NEXT_PUBLIC_CONTRACT_FRAUD_PREVENTION = 'CFRAUD';
    process.env.NEXT_PUBLIC_CONTRACT_GOVERNANCE_DAO = 'CGOV_DAO';
    process.env.NEXT_PUBLIC_CONTRACT_GOVERNANCE_TOKEN = 'CGOV_TOKEN';
    process.env.NEXT_PUBLIC_CONTRACT_IDENTITY_REGISTRY = 'CIDENTITY';
    process.env.NEXT_PUBLIC_CONTRACT_PAYMENT_PROCESSOR = 'CPAYMENT';
    process.env.NEXT_PUBLIC_CONTRACT_PRIVACY_LAYER = 'CPRIVACY';
    process.env.NEXT_PUBLIC_CONTRACT_PUBLISHER_REPUTATION = 'CREP';
    process.env.NEXT_PUBLIC_CONTRACT_PUBLISHER_VERIFICATION = 'CVERIFY';
    process.env.NEXT_PUBLIC_CONTRACT_REVENUE_SETTLEMENT = 'CREVENUE';
    process.env.NEXT_PUBLIC_CONTRACT_REWARDS_DISTRIBUTOR = 'CREWARDS';
    process.env.NEXT_PUBLIC_CONTRACT_SUBSCRIPTION_MANAGER = 'CSUB';
    process.env.NEXT_PUBLIC_CONTRACT_TARGETING_ENGINE = 'CTARGET';

    expect(() => validateRequiredEnv()).not.toThrow();
  });

  it('throws when required env vars are missing', () => {
    delete process.env.NEXT_PUBLIC_NETWORK;
    expect(() => validateRequiredEnv()).toThrow('Missing required environment variable');
    expect(() => validateRequiredEnv()).toThrow('NEXT_PUBLIC_NETWORK');
  });
});

describe('frontend/.env.example divergence check', () => {
  it('ensures frontend/.env.example lists every REQUIRED_ENV_VARS entry', async () => {
    const fs = await import('fs');
    const path = await import('path');
    const envExamplePath = path.resolve(__dirname, '../../.env.example');
    const content = fs.readFileSync(envExamplePath, 'utf-8');
    for (const key of REQUIRED_ENV_VARS) {
      expect(content).toContain(key);
    }
  });
});

