import { SQLDatabase } from "encore.dev/storage/sqldb";

export const db = new SQLDatabase("kageshirei", {
    migrations: "./migrations",
})

export interface IUser {
    /**
     * id is the unique identifier for the user
     */
    id: number;
    /**
     * username is the unique username for the user
     */
    username: string;
    /**
     * password is the hashed (bcrypt) password for the user
     */
    password: string;
    /**
     * created_at is the timestamp when the user was created
     */
    created_at: number;
    /**
     * updated_at is the timestamp when the user was last updated
     */
    updated_at: number;
}