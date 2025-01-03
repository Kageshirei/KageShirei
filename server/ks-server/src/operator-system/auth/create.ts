import { api } from "encore.dev/api";
import { auth } from "~encore/clients";
import { InternalCreateData, InternalCreateResponse } from "./internal-create";

/**
 * Create a new user account
 *
 * This endpoint is publicly exposed and authenticated. Internally this call proxies to the `auth.internalCreate`
 * endpoint.
 */
export const create = api(
    {
        auth:   true,
        expose: true,
        method: "POST",
        path:   "/auth/create",
    },
    async (data: InternalCreateData): Promise<InternalCreateResponse> => {
        return await auth.internalCreate(data);
    },
);
