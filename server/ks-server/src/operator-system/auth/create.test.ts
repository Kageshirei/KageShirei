import { beforeEach, describe, expect, it, Mock, vi } from "vitest";
import { auth } from "~encore/clients";
import { create } from "./create";
import { InternalCreateData, InternalCreateResponse } from "./internal-create";

vi.mock("~encore/clients");

describe("create", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("should create a new user account", async () => {
        const mockResponse: InternalCreateResponse = {username: "newuser"};
        (
            auth.internalCreate as Mock
        ).mockResolvedValue(mockResponse);

        const data: InternalCreateData = {username: "newuser", password: "password", confirm_password: "password"};
        const response = await create(data);

        expect(response).toEqual(mockResponse);
        expect(auth.internalCreate).toHaveBeenCalledWith(data);
    });
});