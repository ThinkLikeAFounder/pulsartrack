declare global {
  namespace Express {
    interface Request {
      stellarAddress?: string;
    }
  }
}

export {};
