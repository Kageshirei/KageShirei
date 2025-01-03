import { db } from "@/database";
import { hashSync } from "bcrypt";
import { APIError } from "encore.dev/api";
import { beforeEach, describe, expect, it, Mock, vi } from "vitest";
import { internalCreate } from "./internal-create";

vi.mock("@/database");
vi.mock("bcrypt");

describe("internal_create", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("should create a new user with provided password", async () => {
        (
            db.queryRow as Mock
        ).mockResolvedValue(null);
        (
            db.exec as Mock
        ).mockResolvedValue(undefined);
        (
            hashSync as Mock
        ).mockReturnValue("hashedPassword");

        const data = {username: "newuser", password: "password", confirm_password: "password"};
        const response = await internalCreate(data);

        expect(response).toEqual({username: "newuser"});
        expect(db.queryRow).toHaveBeenCalledWith(
            [
                "select id\n" +
                "                                              from users\n" +
                "                                              where username = ",
                "",
            ],
            "newuser",
        );
        expect(db.exec).toHaveBeenCalledWith(
            [
                "insert into users (username, password) values (",
                ", ",
                ")",
            ],
            "newuser",
            "hashedPassword",
        );
        expect(hashSync).toHaveBeenCalledWith("password", 10);
    });

    it("should throw an error if username already exists", async () => {
        (
            db.queryRow as Mock
        ).mockResolvedValue({id: 1});

        const data = {username: "existinguser", password: "password", confirm_password: "password"};

        await expect(internalCreate(data)).rejects.toThrow(APIError.aborted("Username already exists"));
    });

    it("should throw an error if passwords do not match", async () => {
        (
            db.queryRow as Mock
        ).mockResolvedValue(null);
        const data = {username: "newuser1", password: "password", confirm_password: "differentpassword"};

        await expect(internalCreate(data)).rejects.toThrow(APIError.aborted("Passwords do not match"));
    });

    it("should throw an error if no password is provided nor requested", async () => {
        (
            db.queryRow as Mock
        ).mockResolvedValue(null);
        const data = {username: "newuser1"};

        await expect(internalCreate(data)).rejects.toThrow(APIError.aborted("Password is required"));
    });

    it("should create a new user with generated password", async () => {
        (
            db.queryRow as Mock
        ).mockResolvedValue(null);
        (
            db.exec as Mock
        ).mockResolvedValue(undefined);
        (
            hashSync as Mock
        ).mockReturnValue("hashedPassword");

        const data = {username: "newuser", generate_password: true};
        const response = await internalCreate(data);

        expect(response.username).toEqual("newuser");
        expect(response.password).toBeDefined();
        expect(db.queryRow).toHaveBeenCalledWith(
            [
                "select id\n" +
                "                                              from users\n" +
                "                                              where username = ",
                "",
            ],
            "newuser",
        );
        expect(db.exec).toHaveBeenCalledWith(
            [
                "insert into users (username, password) values (",
                ", ",
                ")",
            ],
            "newuser",
            "hashedPassword",
        );
        expect(hashSync).toHaveBeenCalledWith(expect.any(String), 10);
    });
});