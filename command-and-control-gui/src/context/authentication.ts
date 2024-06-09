import {dayjs} from "@/helpers/dayjs";
import {notifications} from "@mantine/notifications";
import {AppRouterInstance} from "next/dist/shared/lib/app-router-context.shared-runtime";
import {proxy} from "valtio";

export interface IAuthenticate {
    host: string;
    bearer: string;
    username: string;
    expires_in: number;
}

/**
 * The authentication context used to store the authentication data
 *
 * Note: This does not persist the data, it is only stored in memory, so if the page is refreshed the data will be lost
 */
class Authentication {
    private _elapses_at: dayjs.Dayjs | null = null;
    private _refresh_interval: NodeJS.Timeout | null = null;
    private _expires_in: number = 0;

    constructor() {
        if (!window || !localStorage) {
            return;
        }

        if (localStorage.getItem("auth")) {
            const data = JSON.parse(localStorage.getItem("auth")!);

            // If the data is not valid then return
            if (!data.bearer || !data.host || !data.expires_in || !data.username || data.bearer.length === 0) {
                return;
            }

            this.authenticate(data);
            this.refresh(true);
        }
    }

    private _host: string = "";

    public get host() {
        return this._host;
    }

    private _username: string = "";

    public get username() {
        return this._username;
    }

    private _bearer: string = "";

    public get bearer() {
        return this._bearer;
    }

    private _is_authenticated: boolean = false;

    public get is_authenticated() {
        return this._is_authenticated;
    }

    /**
     * Authenticate the user
     * @param data The authentication data
     */
    public authenticate(data: IAuthenticate) {
        console.log("Authenticating", data);

        // If the data is not valid then return
        if (data.bearer.length === 0) {
            return;
        }

        window.localStorage.setItem("auth", JSON.stringify(data));

        // Clear the interval if it exists
        if (this._refresh_interval) {
            clearInterval(this._refresh_interval);
        }

        this._host = data.host;
        this._expires_in = data.expires_in;
        this._bearer = data.bearer;
        this._username = data.username;
        this._is_authenticated = true;

        // Set the elapses_at to the current time plus the expires_in minus 1 minute to ensure enough time to refresh
        // the token before it expires
        this._elapses_at = dayjs.utc().add(this._expires_in, "second").subtract(1, "minute");
        this._refresh_interval = setInterval(this.refresh.bind(this), 5000);
    }

    /**
     * Logout the user
     * @param {AppRouterInstance} router
     */
    public logout(router: AppRouterInstance) {
        localStorage.removeItem("auth");

        // Clear the interval if it exists
        if (this._refresh_interval) {
            clearInterval(this._refresh_interval);
        }

        this._host = "";
        this._expires_in = 0;
        this._bearer = "";
        this._username = "";
        this._is_authenticated = false;
        this._elapses_at = null;

        // Redirect the user to the login page
        router.push("/");
    }

    /**
     * Refresh the token
     * @returns {Promise<void>}
     * @private
     */
    private async refresh(bypass_time_check = false): Promise<void> {
        // if the elapsed time is before the current time then the token is about to expire and we need to refresh
        // it
        if (this._elapses_at!.isBefore(dayjs.utc()) || bypass_time_check) {
            console.log("Token about to expire, refreshing");

            // send a request to refresh the token
            const response = await fetch(`http://${this._host}/refresh-token`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "Authorization": `Bearer ${this._bearer}`,
                },
            });

            // if the response is not ok then show an error notification
            if (!response.ok) {
                notifications.show({
                    title: "Failed to refresh token",
                    message: (await response.json()).error,
                    color: "red",
                });
                return;
            }

            // get the new data
            const data = await response.json();
            this._bearer = data.token;
            this._expires_in = data.expires_in;

            // update the local storage
            localStorage.setItem("auth", JSON.stringify({
                host: this._host,
                bearer: this._bearer,
                expires_in: this._expires_in,
                username: this._username,
            }));

            // and update the elapses_at
            this._elapses_at = dayjs.utc().add(this._expires_in, "second").subtract(1, "minute");
        }
    }
}

export const AuthenticationCtx = proxy(new Authentication());