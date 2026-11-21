/** Safely extract a message from a caught value of unknown type. */
export function errorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "object" && err !== null && "message" in err) {
    return String((err as { message: unknown }).message);
  }
  return String(err);
}

/** Read a string `code` property (e.g. Prisma/Node error codes) from an unknown error. */
export function errorCode(err: unknown): string | undefined {
  if (typeof err === "object" && err !== null && "code" in err) {
    const code = (err as { code: unknown }).code;
    return typeof code === "string" ? code : undefined;
  }
  return undefined;
}

/** Read the HTTP status from an axios-style `response.status` on an unknown error. */
export function errorHttpStatus(err: unknown): number | undefined {
  if (typeof err === "object" && err !== null && "response" in err) {
    const response = (err as { response?: { status?: unknown } }).response;
    return typeof response?.status === "number" ? response.status : undefined;
  }
  return undefined;
}
