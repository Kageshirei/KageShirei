import { GlobalLogs } from "@/context/globals/logs";
import { GlobalSessions } from "@/context/globals/sessions";
import { all } from "radash";

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

    await all([
        initAgents(host, bearer),
        initLogs(host, bearer),
    ]);
}

/**
 * Initialize the logs.
 * @param {string} host
 * @param {string} bearer
 * @returns {Promise<void>}
 */
async function initLogs(host: string, bearer: string): Promise<void> {
    GlobalLogs.is_fetching = true;

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
        GlobalLogs.data.push(...data);
    }
    catch (error) {
        console.error("Failed to fetch logs", error);
    }

    GlobalLogs.is_fetching = false;
}

/**
 * Initialize the agents.
 * @param {string} host
 * @param {string} bearer
 * @returns {Promise<void>}
 */
async function initAgents(host: string, bearer: string): Promise<void> {
    GlobalSessions.is_fetching = true;

    try {
        const response = await fetch(`http://${ host }/sessions`, {
            method:  "GET",
            headers: {
                Authorization: `Bearer ${ bearer }`,
            },
        });

        if (!response.ok) {
            throw new Error(`Failed to fetch agents: ${ response.statusText }`);
        }

        const data = await response.json();
        GlobalSessions.data.push(...data);
    }
    catch (error) {
        console.error("Failed to fetch agents", error);
    }

    GlobalSessions.is_fetching = false;
}