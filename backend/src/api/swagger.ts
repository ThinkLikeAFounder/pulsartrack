import swaggerJSDoc from "swagger-jsdoc";
import swaggerUi from "swagger-ui-express";

const swaggerDefinition = {
  openapi: "3.0.3",
  info: {
    title: "PulsarTrack API",
    version: "1.0.0",
    description:
      "Backend API for PulsarTrack. Use this spec to explore endpoints and payloads.",
  },
  servers: [
    {
      url: "/",
    },
  ],
  components: {
    securitySchemes: {
      bearerAuth: {
        type: "http",
        scheme: "bearer",
        bearerFormat: "JWT",
      },
    },
    schemas: {
      ErrorResponse: {
        type: "object",
        properties: {
          error: { type: "string" },
          details: { type: "string" },
        },
      },
      HealthResponse: {
        type: "object",
        properties: {
          status: { type: "string" },
          checks: { type: "object", additionalProperties: { type: "string" } },
          uptime: { type: "number" },
          timestamp: { type: "string", format: "date-time" },
        },
      },
      NetworkResponse: {
        type: "object",
        properties: {
          network: { type: "string" },
          horizonUrl: { type: "string" },
          sorobanRpcUrl: { type: "string" },
          feeStats: { type: "object", additionalProperties: true },
        },
      },
      AccountResponse: {
        type: "object",
        additionalProperties: true,
      },
      AccountTransactionsResponse: {
        type: "object",
        properties: {
          transactions: { type: "array", items: { type: "object" } },
          count: { type: "number" },
        },
      },
      ContractsResponse: {
        type: "object",
        properties: {
          contracts: { type: "object", additionalProperties: { type: "string" } },
        },
      },
      CampaignStatsResponse: {
        type: "object",
        properties: {
          total_campaigns: { type: "number" },
          active_campaigns: { type: "number" },
          total_impressions: { type: "number" },
          total_clicks: { type: "number" },
          total_spent_xlm: { type: "number" },
        },
      },
      CampaignCreateRequest: {
        type: "object",
        required: ["title", "contentId", "budgetStroops", "dailyBudgetStroops"],
        properties: {
          title: { type: "string" },
          contentId: { type: "string" },
          budgetStroops: { type: "number" },
          dailyBudgetStroops: { type: "number" },
        },
      },
      PublisherRegisterRequest: {
        type: "object",
        required: ["displayName"],
        properties: {
          displayName: { type: "string" },
          website: { type: "string" },
        },
      },
      PublisherLeaderboardResponse: {
        type: "object",
        properties: {
          publishers: {
            type: "array",
            items: {
              type: "object",
              properties: {
                address: { type: "string" },
                displayName: { type: "string" },
                tier: { type: "string" },
                reputationScore: { type: "number" },
                impressionsServed: { type: "number" },
                earningsXlm: { type: "number" },
                lastActivity: { type: "string" },
              },
            },
          },
        },
      },
      AuctionListResponse: {
        type: "object",
        properties: {
          auctions: { type: "array", items: { type: "object" } },
          total: { type: "number" },
        },
      },
      BidRequest: {
        type: "object",
        required: ["campaignId", "amountStroops"],
        properties: {
          campaignId: { type: "number" },
          amountStroops: { type: "number" },
        },
      },
      GovernanceProposalsResponse: {
        type: "object",
        properties: {
          proposals: { type: "array", items: { type: "object" } },
          totalOnChain: { type: ["number", "null"] },
        },
      },
    },
  },
};

const swaggerSpec = swaggerJSDoc({
  definition: swaggerDefinition,
  apis: ["./src/**/*.ts", "./dist/**/*.js"],
});

export { swaggerSpec, swaggerUi };
