"use client";

import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useCreateCampaign } from "@/hooks/useContract";
import { campaignSchema, CampaignFormData } from "@/lib/validation/schemas";
import { useState } from "react";

interface CampaignFormProps {
  onSuccess?: (campaignId: number) => void;
  onCancel?: () => void;
}

type CampaignSubmission =
  | { ok: false; error: string }
  | {
      ok: true;
      payload: {
        title: string;
        contentId: string;
        campaignType: number;
        budgetXlm: number;
        costPerViewXlm: number;
        durationDays: number;
        targetViews: number;
        dailyViewLimit: number;
        refundable: boolean;
      };
    };

export function parseCampaignSubmission(
  data: CampaignFormData,
): CampaignSubmission {
  const budget = Number.parseFloat(data.budgetXlm);
  const costPerView = Number.parseFloat(data.costPerViewXlm);
  const campaignType = Number(data.campaignType);
  const durationDays = Number(data.durationDays);
  const targetViews = Number.parseInt(data.targetViews, 10);
  const dailyViewLimit = Number.parseInt(data.dailyViewLimit, 10);

  if (!Number.isFinite(budget) || !Number.isFinite(costPerView)) {
    return { ok: false, error: "Invalid numeric values" };
  }

  return {
    ok: true,
    payload: {
      title: data.title,
      contentId: data.contentId,
      campaignType,
      budgetXlm: budget,
      costPerViewXlm: costPerView,
      durationDays,
      targetViews,
      dailyViewLimit,
      refundable: data.refundable ?? true,
    },
  };
}

export function CampaignForm({ onSuccess, onCancel }: CampaignFormProps) {
  const [submitError, setSubmitError] = useState<string | null>(null);
  const { createCampaign, isPending } = useCreateCampaign();

  const {
    register,
    handleSubmit,
    reset,
    watch,
    formState: { errors },
  } = useForm<CampaignFormData>({
    resolver: zodResolver(campaignSchema),
    mode: "onTouched",
    defaultValues: {
      title: "",
      contentId: "",
      campaignType: 1,
      budgetXlm: "",
      costPerViewXlm: "0.001",
      durationDays: 30,
      targetViews: "10000",
      dailyViewLimit: "1000",
      refundable: true,
    },
  });

  const budgetXlm = watch("budgetXlm");
  const costPerViewXlm = watch("costPerViewXlm");

  // Calculate suggested target views based on budget and cost per view
  const suggestedTargetViews =
    budgetXlm && costPerViewXlm
      ? Math.floor(parseFloat(budgetXlm) / parseFloat(costPerViewXlm))
      : 0;

  const onSubmit = async (data: CampaignFormData) => {
    setSubmitError(null);

    try {
      const submission = parseCampaignSubmission(data);

      if (!submission.ok) {
        setSubmitError(submission.error);
        return;
      }

      const result = await createCampaign(submission.payload);

      if (!result.success) {
        throw new Error(result.error || "Failed to create campaign");
      }

      const campaignId = Number(result.result);
      if (!Number.isSafeInteger(campaignId) || campaignId <= 0) {
        throw new Error("Invalid campaign ID returned");
      }

      onSuccess?.(campaignId);
      reset();
    } catch (err: unknown) {
      setSubmitError(
        err instanceof Error ? err.message : "Failed to create campaign",
      );
    }
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
      <div>
        <label
          htmlFor="campaign-title"
          className="block text-sm font-medium text-gray-300 mb-1"
        >
          Campaign Title <span className="text-red-400">*</span>
        </label>
        <input
          id="campaign-title"
          type="text"
          {...register("title")}
          placeholder="My Campaign"
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
        />
        {errors.title && (
          <p className="text-red-400 text-xs mt-1">{errors.title.message}</p>
        )}
      </div>

      <div>
        <label
          htmlFor="campaign-content-id"
          className="block text-sm font-medium text-gray-300 mb-1"
        >
          Content ID <span className="text-red-400">*</span>
        </label>
        <input
          id="campaign-content-id"
          type="text"
          {...register("contentId")}
          placeholder="ipfs://..."
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
        />
        {errors.contentId && (
          <p className="text-red-400 text-xs mt-1">{errors.contentId.message}</p>
        )}
      </div>

      <div>
        <label
          htmlFor="campaign-type"
          className="block text-sm font-medium text-gray-300 mb-1"
        >
          Campaign Type <span className="text-red-400">*</span>
        </label>
        <select
          id="campaign-type"
          {...register("campaignType", { valueAsNumber: true })}
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-indigo-500 text-sm"
        >
          <option value={1}>Standard</option>
          <option value={2}>Premium</option>
          <option value={3}>Enterprise</option>
        </select>
        {errors.campaignType && (
          <p className="text-red-400 text-xs mt-1">
            {errors.campaignType.message}
          </p>
        )}
      </div>

      <div className="grid grid-cols-2 gap-3">
        <div>
          <label
            htmlFor="campaign-budget"
            className="block text-sm font-medium text-gray-300 mb-1"
          >
            Total Budget (XLM) <span className="text-red-400">*</span>
          </label>
          <input
            id="campaign-budget"
            type="number"
            {...register("budgetXlm")}
            placeholder="500"
            min="0.1"
            step="0.1"
            className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
          />
          {errors.budgetXlm && (
            <p className="text-red-400 text-xs mt-1">
              {errors.budgetXlm.message}
            </p>
          )}
        </div>
        <div>
          <label
            htmlFor="campaign-cost-per-view"
            className="block text-sm font-medium text-gray-300 mb-1"
          >
            Cost Per View (XLM) <span className="text-red-400">*</span>
          </label>
          <input
            id="campaign-cost-per-view"
            type="number"
            {...register("costPerViewXlm")}
            placeholder="0.001"
            min="0.0001"
            step="0.0001"
            className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
          />
          {errors.costPerViewXlm && (
            <p className="text-red-400 text-xs mt-1">
              {errors.costPerViewXlm.message}
            </p>
          )}
        </div>
      </div>

      <div className="grid grid-cols-2 gap-3">
        <div>
          <label
            htmlFor="campaign-target-views"
            className="block text-sm font-medium text-gray-300 mb-1"
          >
            Target Views <span className="text-red-400">*</span>
          </label>
          <input
            id="campaign-target-views"
            type="number"
            {...register("targetViews")}
            placeholder={
              suggestedTargetViews > 0
                ? suggestedTargetViews.toString()
                : "10000"
            }
            min="1"
            step="1"
            className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
          />
          {errors.targetViews && (
            <p className="text-red-400 text-xs mt-1">
              {errors.targetViews.message}
            </p>
          )}
          {suggestedTargetViews > 0 && (
            <p className="text-gray-400 text-xs mt-1">
              Suggested: {suggestedTargetViews.toLocaleString()} views
            </p>
          )}
        </div>
        <div>
          <label
            htmlFor="campaign-daily-limit"
            className="block text-sm font-medium text-gray-300 mb-1"
          >
            Daily View Limit <span className="text-red-400">*</span>
          </label>
          <input
            id="campaign-daily-limit"
            type="number"
            {...register("dailyViewLimit")}
            placeholder="1000"
            min="1"
            step="1"
            className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
          />
          {errors.dailyViewLimit && (
            <p className="text-red-400 text-xs mt-1">
              {errors.dailyViewLimit.message}
            </p>
          )}
        </div>
      </div>

      <div>
        <label
          htmlFor="campaign-duration"
          className="block text-sm font-medium text-gray-300 mb-1"
        >
          Duration (days) <span className="text-red-400">*</span>
        </label>
        <select
          id="campaign-duration"
          {...register("durationDays", { valueAsNumber: true })}
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-indigo-500 text-sm"
        >
          {[7, 14, 30, 60, 90].map((d) => (
            <option key={d} value={d}>
              {d} days
            </option>
          ))}
        </select>
        {errors.durationDays && (
          <p className="text-red-400 text-xs mt-1">
            {errors.durationDays.message}
          </p>
        )}
      </div>

      <div className="flex items-center">
        <input
          id="campaign-refundable"
          type="checkbox"
          {...register("refundable")}
          className="w-4 h-4 text-indigo-600 bg-gray-700 border-gray-600 rounded focus:ring-indigo-500 focus:ring-2"
        />
        <label
          htmlFor="campaign-refundable"
          className="ml-2 text-sm text-gray-300"
        >
          Allow refund of remaining budget if campaign is cancelled
        </label>
      </div>

      {submitError && (
        <div className="bg-red-900/30 border border-red-700 rounded-lg px-3 py-2 text-red-300 text-sm">
          {submitError}
        </div>
      )}

      <div className="flex gap-3 pt-2">
        <button
          type="submit"
          disabled={isPending}
          className="flex-1 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium py-2 px-4 rounded-lg transition-colors text-sm"
        >
          {isPending ? "Creating..." : "Create Campaign"}
        </button>
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            className="px-4 py-2 border border-gray-600 text-gray-300 rounded-lg hover:bg-gray-700 transition-colors text-sm"
          >
            Cancel
          </button>
        )}
      </div>
    </form>
  );
}
