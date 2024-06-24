export interface Agent {
    /**
     * The unique identifier for the agent (cuid2)
     */
    id: string;
    /**
     * The OS name
     */
    operative_system: string;
    /**
     * The victim hostname
     */
    hostname: string;
    /**
     * The domain of the victim
     */
    domain: string;
    /**
     * The username of whose runs the agent
     */
    username: string;
    /**
     * The internal IP of the victim
     */
    ip: string;
    /**
     * The process ID of the agent
     */
    process_id: number;
    /**
     * The parent process ID of the agent
     */
    parent_process_id: number;
    /**
     * The process name of the agent
     */
    process_name: string;
    /**
     * Whether the agent is running as elevated
     */
    elevated: boolean;
    /**
     * The current working directory of the agent
     */
    cwd: string;
}
