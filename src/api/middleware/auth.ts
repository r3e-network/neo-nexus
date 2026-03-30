import jwt from "jsonwebtoken";
import type { SignOptions } from "jsonwebtoken";
import { Request, Response, NextFunction } from "express";
import { ApiError, Errors } from "../errors";

function getJwtSecret(): string {
  const secret = process.env.JWT_SECRET;
  if (!secret && process.env.NODE_ENV === "production") {
    throw new Error("JWT_SECRET environment variable is required in production");
  }
  return secret || "dev-only-secret-" + Math.random().toString(36);
}

const JWT_SECRET = getJwtSecret();
const JWT_EXPIRES_IN: SignOptions["expiresIn"] = (process.env.JWT_EXPIRES_IN || "24h") as SignOptions["expiresIn"];

export interface JwtPayload {
  userId: string;
  username: string;
}

export interface SessionUser {
  id: string;
  username: string;
  role: "admin" | "viewer";
}

export interface AuthenticatedRequest extends Request {
  user: SessionUser;
}

interface SessionVerifier {
  verifySession(token: string): SessionUser | null;
}

export function generateToken(payload: JwtPayload): string {
  return jwt.sign(payload, JWT_SECRET, { expiresIn: JWT_EXPIRES_IN });
}

export function verifyToken(token: string): JwtPayload {
  return jwt.verify(token, JWT_SECRET) as JwtPayload;
}

function getBearerToken(req: Request): string | null {
  const authHeader = req.headers.authorization;

  if (!authHeader?.startsWith("Bearer ")) {
    return null;
  }

  return authHeader.substring(7);
}

const sendError = (res: Response, err: ApiError) =>
  res.status(err.status).json({ error: err.message, code: err.code, suggestion: err.suggestion, status: err.status });

export function createAuthMiddleware(sessionVerifier: SessionVerifier) {
  return function sessionAuthMiddleware(req: Request, res: Response, next: NextFunction) {
    const token = getBearerToken(req);

    if (!token) {
      return sendError(res, Errors.noToken());
    }

    try {
      const payload = verifyToken(token);
      const sessionUser = sessionVerifier.verifySession(token);

      if (!sessionUser || payload.userId !== sessionUser.id || payload.username !== sessionUser.username) {
        return sendError(res, Errors.sessionInvalid());
      }

      (req as AuthenticatedRequest).user = sessionUser;
      next();
    } catch {
      return sendError(res, Errors.tokenInvalid());
    }
  };
}
