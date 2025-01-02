import {secret} from "encore.dev/config";

/**
 * JWT_SECRET is the secret used to sign JWT tokens
 */
export const JWT_SECRET = secret("JWT_SECRET")