import { ILog } from "@/interfaces/log";
import { proxy } from "valtio";

interface IGlobalLogs {
    /**
     * The logs data
     */
    data: ILog[];
    /**
     * The latest page fetched
     */
    page: number;
    /**
     * Whether the logs are being fetched
     */
    is_fetching: boolean;
}

/**
 * The global logs.
 */
export const GlobalLogs = proxy<IGlobalLogs>({
    data:        [],
    page:        1,
    is_fetching: false,
});