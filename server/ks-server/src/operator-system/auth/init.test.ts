import { db } from "@/database";
import { APIError } from "encore.dev/api";
import { beforeEach, describe, expect, it, Mock, vi } from "vitest";
import { auth } from "~encore/clients";
import { init } from "./init";

vi.mock("@/database");
vi.mock("~encore/clients");

describe("init", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("should initialize the server with a new user if no users exist", async () => {
        (
            db.queryRow as Mock
        ).mockResolvedValue({count: 0});
        (
            auth.internalCreate as Mock
        ).mockResolvedValue({userID: "1"});

        const data = {username: "newuser", password: "newpassword"};
        const response = await init(data);

        expect(response).toEqual({userID: "1"});
        expect(db.queryRow).toHaveBeenCalledWith(expect.anything());
        expect(auth.internalCreate).toHaveBeenCalledWith(data);
    });

    it("should throw an error if users already exist", async () => {
        (
            db.queryRow as Mock
        ).mockResolvedValue({count: 1});

        const data = {username: "newuser", password: "newpassword"};

        await expect(init(data))
            .rejects
            .toThrow(APIError.permissionDenied("Cannot initialize server, users already exist"));
    });
});