import { z } from "zod";

const VALID_DURATIONS = [7, 14, 30, 60, 90] as const;

export const campaignSchema = z
  .object({
    title: z.string().min(1, "Title is required").max(200, "Title must be 200 characters or less"),
    contentId: z.string().min(1, "Content ID is required"),
    campaignType: z.coerce
      .number()
      .int("Campaign type must be a whole number")
      .min(1, "Campaign type is required"),
    budgetXlm: z
      .string()
      .min(1, "Budget is required")
      .refine((v) => !isNaN(parseFloat(v)) && parseFloat(v) > 0, {
        message: "Budget must be a positive number",
      }),
    costPerViewXlm: z
      .string()
      .min(1, "Cost per view is required")
      .refine((v) => !isNaN(parseFloat(v)) && parseFloat(v) > 0, {
        message: "Cost per view must be a positive number",
      }),
    durationDays: z.coerce
      .number()
      .refine((v) => (VALID_DURATIONS as readonly number[]).includes(v), {
        message: "Select a valid duration",
      }),
    targetViews: z
      .string()
      .min(1, "Target views is required")
      .refine((v) => !isNaN(parseInt(v)) && parseInt(v) > 0, {
        message: "Target views must be a positive number",
      }),
    dailyViewLimit: z
      .string()
      .min(1, "Daily view limit is required")
      .refine((v) => !isNaN(parseInt(v)) && parseInt(v) > 0, {
        message: "Daily view limit must be a positive number",
      }),
    refundable: z.boolean().default(true),
  })
  .refine(
    (data) => {
      const budget = parseFloat(data.budgetXlm);
      const costPerView = parseFloat(data.costPerViewXlm);
      const targetViews = parseInt(data.targetViews);
      if (isNaN(budget) || isNaN(costPerView) || isNaN(targetViews))
        return true;
      const totalCost = costPerView * targetViews;
      return budget >= totalCost;
    },
    {
      message:
        "Budget must be sufficient for target views (budget >= cost per view ? target views)",
      path: ["budgetXlm"],
    },
  )
  .refine(
    (data) => {
      const targetViews = parseInt(data.targetViews);
      const dailyLimit = parseInt(data.dailyViewLimit);
      if (isNaN(targetViews) || isNaN(dailyLimit)) return true;
      return dailyLimit <= targetViews;
    },
    {
      message: "Daily view limit cannot exceed target views",
      path: ["dailyViewLimit"],
    },
  );

export type CampaignFormData = z.input<typeof campaignSchema>;

export function createBidSchema(minBid: number) {
  return z.object({
    campaignId: z
      .string()
      .min(1, "Campaign ID is required")
      .refine((v) => !isNaN(parseInt(v)) && parseInt(v) > 0, {
        message: "Campaign ID must be a positive integer",
      }),
    bidAmountXlm: z
      .string()
      .min(1, "Bid amount is required")
      .refine((v) => !isNaN(parseFloat(v)) && parseFloat(v) > 0, {
        message: "Bid amount must be a positive number",
      })
      .refine((v) => !isNaN(parseFloat(v)) && parseFloat(v) >= minBid, {
        message: `Minimum bid is ${minBid.toFixed(4)} XLM`,
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
      .int("Min age must be a whole number")
      .min(13, "Min age must be at least 13")
      .max(100, "Min age must be at most 100"),
    maxAge: z.coerce
      .number()
      .int("Max age must be a whole number")
      .min(13, "Max age must be at least 13")
      .max(100, "Max age must be at most 100"),
    minReputation: z.coerce
      .number()
      .int("Reputation must be a whole number")
      .min(0, "Reputation must be at least 0")
      .max(1000, "Reputation must be at most 1000"),
    requireKyc: z.boolean(),
    excludeFraud: z.boolean(),
    maxCpmXlm: z.string(),
  })
  .refine((data) => data.maxAge >= data.minAge, {
    message: "Max age must be greater than or equal to min age",
    path: ["maxAge"],
  })
  .refine(
    (data) => {
      if (!data.maxCpmXlm) return true;
      const val = parseFloat(data.maxCpmXlm);
      return !isNaN(val) && val > 0;
    },
    { message: "Max CPM must be a positive number", path: ["maxCpmXlm"] },
  );

export type TargetingFormData = z.input<typeof targetingSchema>;
