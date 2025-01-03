import { db } from "@/database";
import { APIError } from "encore.dev/api";
import jwt from "jsonwebtoken";
import { beforeEach, describe, expect, it, Mock, vi } from "vitest";
import { authenticationHandler } from "./authentication-handler";
import { JWT_SECRET } from "./encore.service";

vi.mock("@/database");
vi.mock("jsonwebtoken");
vi.mock("./encore.service");

describe("authenticationHandler", () => {
    const mockUser = {
        id:       1,
        username: "testuser",
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("should return user data for a valid token", async () => {
        (
            JWT_SECRET as Mock
        ).mockReturnValue("base64EncodedSecret");
        (
            jwt.verify as Mock
        ).mockReturnValue({sub: "1"});
        (
            db.queryRow as Mock
        ).mockResolvedValue(mockUser);

        const params = {authorization: "Bearer validToken"};
        const response = await authenticationHandler(params);

        expect(response).toEqual({userID: "1", username: "testuser"});
        expect(jwt.verify).toHaveBeenCalledWith(
            "validToken",
            Buffer.from("base64EncodedSecret", "base64"),
            {
                algorithms: [ "HS512" ],
                audience:   "kageshirei",
                issuer:     "kageshirei",
            },
        );
        expect(db.queryRow).toHaveBeenCalledWith(
            [
                "SELECT *\n" +
                "                                                          FROM users\n" +
                "                                                          WHERE id = ",
                "",
            ],
            1,
        );
    });

    it("should throw an error for a missing Authorization header", async () => {
        const params = {authorization: ""};

        await expect(authenticationHandler(params))
            .rejects
            .toThrow(APIError.unauthenticated("Missing or invalid Authorization header"));
    });

    it("should throw an error for an invalid token", async () => {
        (
            JWT_SECRET as Mock
        ).mockReturnValue("base64EncodedSecret");
        (
            jwt.verify as Mock
        ).mockImplementation(() => {
            throw new Error("Invalid token");
        });

        const params = {authorization: "Bearer invalidToken"};

        await expect(authenticationHandler(params)).rejects.toThrow(APIError.unauthenticated("Invalid JWT token"));
    });

    it("should throw an error if the user does not exist", async () => {
        (
            JWT_SECRET as Mock
        ).mockReturnValue("base64EncodedSecret");
        (
            jwt.verify as Mock
        ).mockReturnValue({sub: "1"});
        (
            db.queryRow as Mock
        ).mockResolvedValue(null);

        const params = {authorization: "Bearer validToken"};

        await expect(authenticationHandler(params)).rejects.toThrow(APIError.unauthenticated("Invalid user ID"));
    });

    it("should throw an error if the JWT is formatted wrongly", async () => {
        (
            JWT_SECRET as Mock
        ).mockReturnValue("base64EncodedSecret");
        (
            jwt.verify as Mock
        ).mockReturnValue({sub: "this is a string"});
        (
            db.queryRow as Mock
        ).mockResolvedValue(null);

        const params = {authorization: "Bearer validToken"};

        await expect(authenticationHandler(params)).rejects.toThrow(APIError.unauthenticated("Invalid JWT token"));
    });
});