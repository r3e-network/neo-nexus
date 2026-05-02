import type { NextFunction, Request, Response } from "express";
import { Errors } from "../errors";
import { respondWithApiError } from "../respond";
import type { AuthenticatedRequest } from "./auth";

const READ_ONLY_METHODS = new Set(["GET", "HEAD", "OPTIONS"]);

export function requireAdmin(req: Request, res: Response, next: NextFunction) {
  const user = (req as AuthenticatedRequest).user;
  if (!user || user.role !== "admin") {
    return respondWithApiError(res, Errors.adminRequired());
  }
  next();
}

export function requireAdminForUnsafeMethods(req: Request, res: Response, next: NextFunction) {
  if (READ_ONLY_METHODS.has(req.method.toUpperCase())) {
    return next();
  }
  return requireAdmin(req, res, next);
}
