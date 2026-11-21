import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import NetworkMismatchBanner from './NetworkMismatchBanner';
import { useWalletStore } from '../../store/wallet-store';

vi.mock('../../store/wallet-store', () => ({ useWalletStore: vi.fn() }));

const wallet = (state: Record<string, unknown>) =>
    vi.mocked(useWalletStore).mockReturnValue({
        isConnected: true,
        networkMismatch: false,
        freighterNetwork: null,
        network: 'testnet',
        ...state,
    } as unknown as ReturnType<typeof useWalletStore>);

describe('NetworkMismatchBanner', () => {
    beforeEach(() => vi.clearAllMocks());

    it('warns when the wallet is on another network', () => {
        wallet({ networkMismatch: true, freighterNetwork: 'PUBLIC' });
        render(<NetworkMismatchBanner />);

        const alert = screen.getByRole('alert');
        expect(alert).toHaveTextContent('your Freighter wallet is on PUBLIC');
        expect(alert).toHaveTextContent('this app uses testnet');
    });

    it('stays hidden when the networks match', () => {
        wallet({ networkMismatch: false });
        render(<NetworkMismatchBanner />);

        expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    });

    it('stays hidden while no wallet is connected', () => {
        wallet({ isConnected: false, networkMismatch: true });
        render(<NetworkMismatchBanner />);

        expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    });
});
