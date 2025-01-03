import { api, APIError } from "encore.dev/api";
import jwt from "jsonwebtoken";
import { getAuthData } from "~encore/auth";
import { JWT_SECRET } from "./encore.service";

/**
 * Represents the response sent by the login endpoint.
 */
interface LoginResponse {
    /**
     * Bearer token to be used for authentication.
     */
    token: string;
}

/**
 * Endpoint to refresh the token.
 */
export const refresh = api(
    {
        auth:   true,
        expose: true,
        method: "PATCH",
        path:   "/auth/refresh",
    },
    async (): Promise<LoginResponse> => {
        const auth_data = getAuthData();

        if (!auth_data) {
            throw APIError.permissionDenied("Cannot refresh token");
        }

        return {
            token: jwt.sign({}, Buffer.from(JWT_SECRET(), "base64"), {
                algorithm: "HS512",
                audience:  "kageshirei",
                issuer:    "kageshirei",
                subject:   auth_data.userID,
                expiresIn: "15m",
            }),
        };
    },
);
