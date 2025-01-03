import { db } from "@/database";
import { api, APIError } from "encore.dev/api";
import { auth } from "~encore/clients";
import { InternalCreateData, InternalCreateResponse } from "./internal-create";

/**
 * Initializes the server with a new user.
 *
 * This can be called only if no users exist in the system, only one user can be created this way.
 *
 * This is NOT authenticated (as there are no users yet).
 *
 * This internally proxies the call to `auth.internalCreate` endpoint.
 */
export const init = api(
    {
        expose: true,
        method: "POST",
        path:   "/auth/init",
    },
    async (data: InternalCreateData): Promise<InternalCreateResponse> => {
        // Check if there are any users in the system counting the number of users in the database
        const users = await db.queryRow<{
            count: number
        }>`select count(*) as count
           from users`;

        // If there are users, we cannot initialize the server (as we don't want to overwrite existing users)
        if (!users || users.count > 0) {
            throw APIError.permissionDenied("Cannot initialize server, users already exist");
        }

        return await auth.internalCreate(data);
    },
);
