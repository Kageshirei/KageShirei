import { ILog } from "@/interfaces/log";
import { proxy } from "valtio";

/**
 * The global logs.
 */
export const GlobalLogs = proxy([] as ILog[]);
export const GlobalLogsParams = proxy({
    page: 1,
});

/**
 * Whether the global logs have been initialized.
 */
let has_been_initialized = false;

/**
 * Initialize the global logs.
 */
export async function globals_init(host: string, bearer: string) {
    if (has_been_initialized) {
        throw new Error("Globals have already been initialized");
    }

    has_been_initialized = true;

    await initLogs(host, bearer);
}

async function initLogs(host: string, bearer: string) {
    try {
        const response = await fetch(`http://${ host }/logs?page=1`, {
            method:  "GET",
            headers: {
                Authorization: `Bearer ${ bearer }`,
            },
        });

        if (!response.ok) {
            throw new Error(`Failed to fetch logs: ${ response.statusText }`);
        }

        const data = await response.json();
        GlobalLogs.push(...data);
    }
    catch (error) {
        console.error("Failed to fetch logs", error);
    }
}