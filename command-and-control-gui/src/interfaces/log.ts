export interface ILog {
    /**
     * The timestamp of the log entry. This is a Unix timestamp in seconds. UTC.
     */
    timestamp: number;
    /**
     * Extra data to be included in the log entry. This is a key-value pair.
     */
    extra: {
        [key: string]: string;
    };
    /**
     * The log level.
     */
    level: "INFO" | "WARN" | "ERROR" | "DEBUG" | "TRACE";
    /**
     * The title of the log entry. This is a short description of the log entry.
     */
    title?: string;
    /**
     * The message of the log entry. This is a detailed description of the log entry.
     */
    message?: string;
}