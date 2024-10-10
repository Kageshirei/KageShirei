import { Agent } from "@/interfaces/agent";
import { proxy } from "valtio";

interface IGlobalSessions {
    /**
     * The sessions data
     */
    data: Agent[];
    /**
     * Whether the sessions are being fetched
     */
    is_fetching: boolean;

}

/**
 * The global sessions object.
 */
export const GlobalSessions = proxy<IGlobalSessions>({
    data:        [],
    is_fetching: false,
});