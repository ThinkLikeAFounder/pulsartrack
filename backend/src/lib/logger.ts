import pino from "pino";

// Define the transport based on the environment
const transport =
  process.env.NODE_ENV === "production"
    ? undefined // Default structured JSON output for production
    : {
        target: "pino-pretty",
        options: {
          colorize: true,
          translateTime: "SYS:standard",
          ignore: "pid,hostname",
        },
      };

export const logger = pino({
  level: process.env.LOG_LEVEL || "info", // Default to info level
  transport,
});
