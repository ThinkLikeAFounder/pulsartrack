import { z } from 'zod';

const VALID_DURATIONS = [7, 14, 30, 60, 90] as const;

export const campaignSchema = z
    .object({
        title: z
            .string()
            .trim()
            .min(1, 'Title is required')
            .max(100, 'Title must be 100 characters or fewer'),
        contentId: z.string().trim().min(1, 'Content ID is required'),
        budgetXlm: z
            .string()
            .min(1, 'Budget is required')
            .refine((v) => !isNaN(parseFloat(v)) && parseFloat(v) > 0, {
                message: 'Budget must be a positive number',
            }),
        dailyBudgetXlm: z.string(),
        durationDays: z.coerce
            .number()
            .refine((v) => (VALID_DURATIONS as readonly number[]).includes(v), {
                message: 'Select a valid duration',
            }),
        targetGeo: z.string(),
        targetInterests: z.string(),
    })
    .refine(
        (data) => {
            if (!data.dailyBudgetXlm) return true;
            const daily = parseFloat(data.dailyBudgetXlm);
            return !isNaN(daily) && daily > 0;
        },
        { message: 'Daily budget must be a positive number', path: ['dailyBudgetXlm'] }
    )
    .refine(
        (data) => {
            if (!data.dailyBudgetXlm) return true;
            const daily = parseFloat(data.dailyBudgetXlm);
            const total = parseFloat(data.budgetXlm);
            if (isNaN(daily) || isNaN(total)) return true;
            return daily <= total;
        },
        { message: 'Daily budget cannot exceed total budget', path: ['dailyBudgetXlm'] }
    );

export type CampaignFormData = z.input<typeof campaignSchema>;

export function createBidSchema(minBid: number) {
    return z.object({
        campaignId: z
            .string()
            .min(1, 'Campaign ID is required')
            .refine((v) => !isNaN(parseInt(v)) && parseInt(v) > 0, {
                message: 'Campaign ID must be a positive integer',
            }),
        bidAmountXlm: z
            .string()
            .min(1, 'Bid amount is required')
            .refine((v) => !isNaN(parseFloat(v)) && parseFloat(v) > 0, {
                message: 'Bid amount must be a positive number',
            })
            .refine((v) => !isNaN(parseFloat(v)) && parseFloat(v) >= minBid, {
                message: `Bid must be at least ${minBid.toFixed(4)} XLM`,
            }),
    });
}

export type BidFormData = z.input<ReturnType<typeof createBidSchema>>;

export const targetingSchema = z
    .object({
        regions: z.array(z.string()),
        interests: z.array(z.string()),
        excludedSegments: z.array(z.string()),
        devices: z.array(z.string()),
        languages: z.array(z.string()),
        minAge: z.coerce
            .number()
            .int('Min age must be a whole number')
            .min(13, 'Min age must be at least 13')
            .max(100, 'Min age must be at most 100'),
        maxAge: z.coerce
            .number()
            .int('Max age must be a whole number')
            .min(13, 'Max age must be at least 13')
            .max(100, 'Max age must be at most 100'),
        minReputation: z.coerce
            .number()
            .int('Reputation must be a whole number')
            .min(0, 'Reputation must be at least 0')
            .max(1000, 'Reputation must be at most 1000'),
        requireKyc: z.boolean(),
        excludeFraud: z.boolean(),
        maxCpmXlm: z.string(),
    })
    .refine((data) => data.maxAge >= data.minAge, {
        message: 'Max age must be greater than or equal to min age',
        path: ['maxAge'],
    })
    .refine(
        (data) => {
            if (!data.maxCpmXlm) return true;
            const val = parseFloat(data.maxCpmXlm);
            return !isNaN(val) && val > 0;
        },
        { message: 'Max CPM must be a positive number', path: ['maxCpmXlm'] }
    );

export type TargetingFormData = z.input<typeof targetingSchema>;
