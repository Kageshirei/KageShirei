import { db } from "@/database";
import { compareSync } from "bcrypt";
import { api, APIError } from "encore.dev/api";
import jwt from "jsonwebtoken";
import { JWT_SECRET } from "./encore.service";

/**
 * Represents the data sent to the login endpoint.
 */
interface LoginData {
    /**
     * Username of the user.
     */
    username: string;
    /**
     * Password of the user.
     */
    password: string;
}

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
 * Endpoint to login a user.
 */
export const login = api(
    {
        expose: true,
        method: "POST",
        path:   "/auth/login",
    },
    async (data: LoginData): Promise<LoginResponse> => {
        const user = await db.queryRow<{
            id: number,
            password: string
        }>`select id, password
           from users
           where username = ${ data.username }`;

        // If the user does not exist or the password is incorrect, throw an error.
        if (!user || !compareSync(data.password, user.password)) {
            throw APIError.unauthenticated("Wrong username or password");
        }

        return {
            token: jwt.sign({}, Buffer.from(JWT_SECRET(), "base64"), {
                algorithm: "HS512",
                audience:  "kageshirei",
                issuer:    "kageshirei",
                subject:   user.id.toString(),
                expiresIn: "15m",
            }),
        };
    },
);