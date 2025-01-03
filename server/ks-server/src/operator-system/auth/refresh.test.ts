import { APIError } from "encore.dev/api";
import jwt from "jsonwebtoken";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getAuthData } from "~encore/auth";
import { JWT_SECRET } from "./encore.service";
import { refresh } from "./refresh";

vi.mock("~encore/auth");
vi.mock("jsonwebtoken");
vi.mock("./encore.service");

describe("refresh", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("should return a token if auth data is present", async () => {
        (
            getAuthData as vi.Mock
        ).mockReturnValue({userID: "1"});
        (
            JWT_SECRET as vi.Mock
        ).mockReturnValue("base64EncodedSecret");
        (
            jwt.sign as vi.Mock
        ).mockReturnValue("fakeToken");

        const response = await refresh();

        expect(response).toEqual({token: "fakeToken"});
        expect(jwt.sign).toHaveBeenCalledWith(
            {},
            Buffer.from("base64EncodedSecret", "base64"),
            {
                algorithm: "HS512",
                audience:  "kageshirei",
                issuer:    "kageshirei",
                subject:   "1",
                expiresIn: "15m",
            },
        );
    });

    it("should throw an error if auth data is not present", async () => {
        (
            getAuthData as vi.Mock
        ).mockReturnValue(null);

        await expect(refresh()).rejects.toThrow(APIError.permissionDenied("Cannot refresh token"));
    });
});