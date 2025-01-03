import { secret } from "encore.dev/config";
import { Service } from "encore.dev/service";

/**
 * This service is responsible for handling authentication login and basic user management.
 */
export default new Service("auth");

/**
 * JWT_SECRET is the secret used to sign JWT tokens
 */
export const JWT_SECRET = secret("JWT_SECRET");