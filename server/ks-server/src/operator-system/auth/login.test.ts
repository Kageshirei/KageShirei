import { db } from "@/database";
import { compareSync } from "bcrypt";
import { APIError } from "encore.dev/api";
import jwt from "jsonwebtoken";
import { beforeEach, describe, expect, it, Mock, vi } from "vitest";
import { JWT_SECRET } from "./encore.service";
import { login } from "./login";

vi.mock("@/database");
vi.mock("bcrypt");
vi.mock("jsonwebtoken");
vi.mock("./encore.service");

describe("login", () => {
    const mockUser = {
        id:       1,
        password: "hashedPassword",
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("should return a token for valid credentials", async () => {
        (
            db.queryRow as Mock
        ).mockResolvedValue(mockUser);
        (
            compareSync as Mock
        ).mockReturnValue(true);
        (
            JWT_SECRET as Mock
        ).mockReturnValue("base64EncodedSecret");
        (
            jwt.sign as Mock
        ).mockReturnValue("fakeToken");

        const data = {username: "testuser", password: "testpassword"};
        const response = await login(data);

        expect(response).toEqual({token: "fakeToken"});
        expect(db.queryRow).toHaveBeenCalledWith(
            [
                "select id, password\n" +
                "           from users\n" +
                "           where username = ",
                "",
            ],
            "testuser",
        );
        expect(compareSync).toHaveBeenCalledWith("testpassword", "hashedPassword");
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

    it("should throw an error for invalid credentials", async () => {
        (
            db.queryRow as Mock
        ).mockResolvedValue(mockUser);
        (
            compareSync as Mock
        ).mockReturnValue(false);

        const data = {username: "testuser", password: "wrongpassword"};

        await expect(login(data)).rejects.toThrow(APIError.unauthenticated("Wrong username or password"));
    });

    it("should throw an error if user does not exist", async () => {
        (
            db.queryRow as Mock
        ).mockResolvedValue(null);

        const data = {username: "nonexistentuser", password: "testpassword"};

        await expect(login(data)).rejects.toThrow(APIError.unauthenticated("Wrong username or password"));
    });
});