import { db, IUser } from "@/database";
import { APIError, Gateway, Header } from "encore.dev/api";
import { authHandler } from "encore.dev/auth";
import jwt from "jsonwebtoken";
import { JWT_SECRET } from "./encore.service";

// AuthParams specifies the incoming request information
// the auth handler is interested in. In this case it only
// cares about requests that contain the `Authorization` header.
interface AuthParams {
    authorization: Header<"Authorization">;
}

// The AuthData specifies the information about the authenticated user
// that the auth handler makes available.
interface AuthData {
    userID: string,
    username: string;
}

const bearer_prefix = "Bearer ";

// The auth handler itself.
export const authenticationHandler = authHandler<AuthParams, AuthData>(
    async (params) => {
        // Check if the Authorization header is present and has the correct format
        if (params.authorization.substring(0, bearer_prefix.length).toLowerCase() !== bearer_prefix.toLowerCase()) {
            throw APIError.unauthenticated("Missing or invalid Authorization header");
        }

        try {
            // Verify the JWT token
            const decoded_jwt = jwt.verify(
                params.authorization.substring(bearer_prefix.length),
                Buffer.from(JWT_SECRET(), "base64"),
                {
                    algorithms: [ "HS512" ],
                    audience:   "kageshirei",
                    issuer:     "kageshirei",
                },
            );


            // Ensure the decoded JWT token is in the expected format
            if (typeof decoded_jwt !== "object" || !decoded_jwt.sub || !Number.isInteger(+decoded_jwt.sub)) {
                throw APIError.unauthenticated("Invalid JWT token");
            }

            // Check if the user exists in the database
            const current_user = await db.queryRow<IUser>`SELECT *
                                                          FROM users
                                                          WHERE id = ${ +decoded_jwt.sub }`;
            if (!current_user) {
                throw APIError.unauthenticated("Invalid user ID");
            }

            return {userID: current_user.id.toString(), username: current_user.username};
        }
        catch (e) {
            // If the error is an APIError, we can just throw it as is.
            if (e instanceof APIError) {
                throw e;
            }

            throw APIError.unauthenticated("Invalid JWT token");
        }
    },
);

// Define the API Gateway that will execute the auth handler:
export const gateway = new Gateway(
    {
        authHandler: authenticationHandler,
    },
);
