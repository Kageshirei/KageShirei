import {proxy} from "valtio";
import {dayjs} from "@/helpers/dayjs";

export interface IAuthenticate {
    bearer: string;
    username: string;
    expires_in?: number;
    refresh_callback: (() => string);
}

class Authentication {
    private _elapses_at: dayjs.Dayjs | null = null;
    private _refresh_callback: (() => string) | null = null;
    private _refresh_interval: NodeJS.Timeout | null = null;

    private _bearer: string = "";

    get bearer() {
        return this._bearer;
    }

    private _username: string = "";

    get username() {
        return this._username;
    }

    private _is_authenticated: boolean = false;

    get is_authenticated() {
        return this._is_authenticated;
    }

    /**
     * Authenticate the user
     * @param _data The authentication data
     */
    public authenticate(_data: IAuthenticate) {
        const data: Omit<IAuthenticate, "expires_in"> & { expires_in: number } = {
            expires_in: 15 * 60, // 15 minutes
            ..._data
        };

        // Clear the interval if it exists
        if (this._refresh_interval) {
            clearInterval(this._refresh_interval);
        }

        this._bearer = data.bearer;
        this._username = data.username;
        this._is_authenticated = true;
        // Set the elapses_at to the current time plus the expires_in minus 1 minute to ensure enough time to refresh
        // the token before it expires
        this._elapses_at = dayjs.utc().add(data.expires_in, 'second').subtract(1, 'minute');
        this._refresh_callback = data.refresh_callback;

        this._refresh_interval = setInterval(() => {
            // if the elapsed time is before the current time then the token is about to expire and we need to refresh it
            if (this._elapses_at!.isBefore(dayjs.utc())) {
                console.log("Token about to expire, refreshing")

                // Refresh the token calling the callback
                this._bearer = this._refresh_callback!();
                // and update the elapses_at
                this._elapses_at = dayjs.utc().add(data.expires_in, 'second').subtract(1, 'minute');
            }
        }, 5000);
    }
}

export const AuthenticationCtx = proxy(new Authentication())