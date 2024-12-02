export interface Agent {
    /**
     * The unique identifier for the agent (cuid2)
     */
    id: string;
    /**
     * The OS name
     */
    operating_system: string;
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
    network_interfaces: NetworkInterfaceArray;
    /**
     * The process ID of the agent
     */
    pid: number;
    /**
     * The parent process ID of the agent
     */
    ppid: number;
    /**
     * The process name of the agent
     */
    process_name: string;
    /**
     * Current process integrity level
     */
    integrity: string;
    /**
     * The current working directory of the agent
     */
    cwd: string;
}

export interface NetworkInterfaceArray {
    network_interfaces: NetworkInterface[];
}

export interface NetworkInterface {
    name?: string | null,
    address?: string | null,
    dhcp_server?: string | null,
}