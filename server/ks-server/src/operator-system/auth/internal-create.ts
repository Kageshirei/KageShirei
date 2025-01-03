import { db } from "@/database";
import { hashSync } from "bcrypt";
import { api, APIError } from "encore.dev/api";

/**
 * Data sent to the internal_create endpoint.
 */
export interface InternalCreateData {
    /**
     * Username to create.
     */
    username: string;
    /**
     * Password for the user.
     */
    password?: string;
    /**
     * Confirmation password for the user.
     */
    confirm_password?: string;
    /**
     * Whether to generate a password for the user, this is mutually exclusive with the password fields.
     */
    generate_password?: boolean;
}

/**
 * Represents the response from the internal_create endpoint.
 */
export interface InternalCreateResponse {
    /**
     * Username of the created user.
     */
    username: string;
    /**
     * Password of the created user if it was generated.
     */
    password?: string;
}

/**
 * Creates a new user in the database.
 *
 * Internal endpoint, not exposed to the public, requires nothing but its arguments.
 */
export const internalCreate = api(
    {},
    async (data: InternalCreateData): Promise<InternalCreateResponse> => {
        const user_exists = await db.queryRow<{
            id: number
        }>`select id
           from users
           where username = ${ data.username }`;

        // Check if the user already exists
        if (user_exists) {
            throw APIError.aborted("Username already exists");
        }

        // Check if the password is provided or if the generate_password flag is set
        if (!data.generate_password && (
            !data.password || !data.confirm_password
        )) {
            throw APIError.aborted("Password is required");
        }

        // Check if the passwords match if they are provided
        if (!data.generate_password && data.password !== data.confirm_password) {
            throw APIError.aborted("Passwords do not match");
        }

        // Generate a password if the generate_password flag is set (or if no password is provided)
        const password = data.generate_password ? generatePassword() : data.password as string;

        // Insert the user into the database
        await db.exec`insert into users (username, password)
                      values (${ data.username }, ${ hashSync(password, 10) })`;

        return {
            username: data.username,
            password: data.generate_password ? password : undefined,
        };
    },
);

/**
 * Generates a random password.
 */
function generatePassword(
    length: number     = 16,
    characters: string = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!@#[].,;:-_+*/!\\|^?()%$",
) {
    return Array.from(crypto.getRandomValues(new Uint32Array(length)))
                .map((x) => characters[x % characters.length])
                .join("");
}