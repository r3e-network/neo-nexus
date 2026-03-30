import type { Response } from "express";
import { ApiError } from "./errors";

export function respondWithApiError(res: Response, error: unknown): void {
  if (error instanceof ApiError) {
    res.status(error.status).json({
      error: error.message,
      code: error.code,
      suggestion: error.suggestion,
      status: error.status,
    });
    return;
  }

  const message = error instanceof Error ? error.message : String(error);
  let status = 500;

  if (/not found/i.test(message)) {
    status = 404;
  } else if (
    /missing required fields|invalid|already exists|requires|cannot|not available|unsupported/i.test(message)
  ) {
    status = 400;
  }

  res.status(status).json({
    error: message,
    code: "INTERNAL_ERROR",
    suggestion: "An unexpected error occurred. If this persists, check the server logs.",
    status,
  });
}
