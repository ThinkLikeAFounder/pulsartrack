import { describe, it, expect } from 'vitest';
import { enumTag, toCampaign, toPublisher } from './contract-adapters';

const ADDR = 'GABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUVW';

describe('contract adapters', () => {
    it('reads an enum variant however the SDK decoded it', () => {
        expect(enumTag('Active')).toBe('Active');
        expect(enumTag(['Paused'])).toBe('Paused');
        expect(enumTag({ Completed: null })).toBe('Completed');
        expect(enumTag(undefined)).toBe('');
    });

    it('turns an orchestrator campaign into the widget shape', () => {
        const campaign = toCampaign({
            id: 7,
            advertiser: ADDR,
            budget: BigInt(1_000_000_000),
            current_views: BigInt(4_200),
            target_views: BigInt(10_000),
            status: ['Expired'],
            remaining_budget: BigInt(250_000_000),
        } as never);

        expect(campaign.campaign_id).toBe(BigInt(7));
        expect(campaign.title).toBe('Campaign #7');
        expect(campaign.spent).toBe(BigInt(750_000_000));
        expect(campaign.impressions).toBe(BigInt(4_200));
        expect(campaign.clicks).toBe(BigInt(0));
        expect(campaign.status).toBe('Completed');
    });

    it('counts nothing spent when the remaining budget is missing', () => {
        const campaign = toCampaign({ id: 1, budget: 500, status: 'Active' });

        expect(campaign.spent).toBe(BigInt(0));
        expect(campaign.status).toBe('Active');
    });

    it('turns a verification record into a publisher', () => {
        const publisher = toPublisher(ADDR, {
            owner: ADDR,
            status: ['Verified'],
            tier: ['Gold'],
            domain: 'news.example',
            reputation_score: 720,
            total_earnings: BigInt(55_000_000),
            total_impressions: BigInt(12_000),
            join_ledger: 100,
        });

        expect(publisher.display_name).toBe('news.example');
        expect(publisher.website).toBe('https://news.example');
        expect(publisher.status).toBe('Verified');
        expect(publisher.tier).toBe('Gold');
        expect(publisher.earnings_total).toBe(BigInt(55_000_000));
        expect(publisher.impressions_served).toBe(BigInt(12_000));
        expect(publisher.reputation_score).toBe(720);
    });

    it('falls back to the address when the publisher has no domain', () => {
        const publisher = toPublisher(ADDR, { status: 'Pending' });

        expect(publisher.display_name).toBe(ADDR);
        expect(publisher.website).toBe('');
    });
});
