import {Header, Gateway, APIError} from "encore.dev/api";
import { authHandler } from "encore.dev/auth";
import {verify} from "jsonwebtoken";
import {JWT_SECRET} from "../../secrets";
import {db, IUser} from "../../database";

// AuthParams specifies the incoming request information
// the auth handler is interested in. In this case it only
// cares about requests that contain the `Authorization` header.
interface AuthParams {
    authorization: Header<"Authorization">;
}

// The AuthData specifies the information about the authenticated user
// that the auth handler makes available.
interface AuthData {
    id: number;
    username: string;
}

const bearer_prefix = "Bearer ";

// The auth handler itself.
export const auth = authHandler<AuthParams, AuthData>(
    async (params) => {
        if (params.authorization.substring(0, bearer_prefix.length).toLowerCase() !== bearer_prefix.toLowerCase()) {
            return APIError.unauthenticated("Missing or invalid Authorization header");
        }

        try {
            const decoded_jwt = verify(params.authorization.substring(bearer_prefix.length), JWT_SECRET(), {
                algorithms: ["HS512"],
                audience: "kageshirei",
                issuer: "kageshirei",
            })

            if (typeof decoded_jwt !== "object") {
                return APIError.unauthenticated("Invalid JWT token");
            }

            if (!decoded_jwt.sub) {
                return APIError.unauthenticated("Invalid JWT token");
            }

            const current_user = await db.queryRow<IUser>`SELECT id FROM users WHERE id = ${decoded_jwt.sub}`
            if (!current_user) {
                return APIError.unauthenticated("Invalid user ID");
            }

            return {id: current_user.id, username: current_user.username};
        }
        catch (e) {
            return APIError.unauthenticated("Invalid JWT token");
        }
    }
)

// Define the API Gateway that will execute the auth handler:
export const gateway = new Gateway({
    authHandler: auth,
})
