import { Router } from "express";
import { logger } from "../lib/logger";

const router = Router();

// Example route registration
router.get("/", (req, res) => {
	logger.info({ ip: req.ip }, "Root route hit");
	// ...existing code...
});

// If there are route-specific logs, convert them similarly:
// logger.debug(...), logger.warn(...), logger.error(...)

export default router;