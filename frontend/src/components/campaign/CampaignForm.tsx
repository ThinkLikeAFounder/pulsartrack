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

export function CampaignForm({ onSuccess, onCancel }: CampaignFormProps) {
  const [submitError, setSubmitError] = useState<string | null>(null);
  const { createCampaign, isPending } = useCreateCampaign();

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isValid },
  } = useForm<CampaignFormData>({
    resolver: zodResolver(campaignSchema),
    mode: "onTouched",
    defaultValues: {
      title: "",
      contentId: "",
      budgetXlm: "",
      dailyBudgetXlm: "",
      durationDays: 30,
      targetGeo: "",
      targetInterests: "",
    },
  });

  const onSubmit = async (data: CampaignFormData) => {
    setSubmitError(null);
    const budget = parseFloat(data.budgetXlm);

    try {
      const result = await createCampaign({
        title: data.title,
        contentId: data.contentId,
        budgetXlm: budget,
        dailyBudgetXlm: data.dailyBudgetXlm
          ? parseFloat(data.dailyBudgetXlm)
          : budget / 30,
        durationDays: data.durationDays as unknown as number,
      });
      onSuccess?.((result as any)?.result || 0);
      reset();
    } catch (err: any) {
      setSubmitError(err?.message || "Failed to create campaign");
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
          placeholder="e.g. Summer Product Launch"
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
          placeholder="IPFS hash or content identifier"
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
        />
        {errors.contentId && (
          <p className="text-red-400 text-xs mt-1">
            {errors.contentId.message}
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
            min="1"
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
            htmlFor="campaign-daily-budget"
            className="block text-sm font-medium text-gray-300 mb-1"
          >
            Daily Budget (XLM)
          </label>
          <input
            id="campaign-daily-budget"
            type="number"
            {...register("dailyBudgetXlm")}
            placeholder="auto"
            min="1"
            step="0.1"
            className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
          />
          {errors.dailyBudgetXlm && (
            <p className="text-red-400 text-xs mt-1">
              {errors.dailyBudgetXlm.message}
            </p>
          )}
        </div>
      </div>

      <div>
        <label
          htmlFor="campaign-duration"
          className="block text-sm font-medium text-gray-300 mb-1"
        >
          Duration (days)
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

      <div>
        <label
          htmlFor="campaign-geo"
          className="block text-sm font-medium text-gray-300 mb-1"
        >
          Geographic Targets
        </label>
        <input
          id="campaign-geo"
          type="text"
          {...register("targetGeo")}
          placeholder="US,EU,APAC (comma-separated)"
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
        />
      </div>

      <div>
        <label
          htmlFor="campaign-interests"
          className="block text-sm font-medium text-gray-300 mb-1"
        >
          Interest Segments
        </label>
        <input
          id="campaign-interests"
          type="text"
          {...register("targetInterests")}
          placeholder="tech,finance,gaming (comma-separated)"
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
        />
      </div>

      {submitError && (
        <div className="bg-red-900/30 border border-red-700 rounded-lg px-3 py-2 text-red-300 text-sm">
          {submitError}
        </div>
      )}

      <div className="flex gap-3 pt-2">
        <button
          type="submit"
          disabled={!isValid || isPending}
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
