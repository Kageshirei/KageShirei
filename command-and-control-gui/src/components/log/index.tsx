import { LogLine } from "@/components/log/log-line";
import { AuthenticationCtx } from "@/context/authentication";
import {
    GlobalLogs,
    GlobalLogsParams,
} from "@/context/globals";
import { ILog } from "@/interfaces/log";
import {
    Button,
    Loader,
} from "@mantine/core";
import { IconReload } from "@tabler/icons-react";
import {
    FC,
    useState,
} from "react";

interface LogProps {
    logs: readonly ILog[];
}

async function loadOlderLogs() {
    try {
        const response = await fetch(`http://${ AuthenticationCtx.host }/logs?page=${ GlobalLogsParams.page + 1 }`, {
            method:  "GET",
            headers: {
                Authorization: `Bearer ${ AuthenticationCtx.bearer }`,
            },
        });

        if (!response.ok) {
            throw new Error(`Failed to fetch logs: ${ response.statusText }`);
        }

        const data = await response.json();

        if (data.length === 0) {
            return false;
        }

        GlobalLogs.unshift(...data);
    }
    catch (error) {
        console.error("Failed to fetch logs", error);
    }

    GlobalLogsParams.page++;
    return true;
}

export const Log: FC<LogProps> = ({
    logs,
}) => {
    const [ can_load_older_logs, set_can_load_older_logs ] = useState(true);
    const [ is_loading_older_logs, set_is_loading_older_logs ] = useState(false);

    return (
        <div className="w-full p-4 bg-zinc-900 rounded font-mono relative h-full pr-6 text-sm max-h-full whitespace-pre">
            <Button variant={ "light" }
                    disabled={ !can_load_older_logs }
                    onClick={ async () => {
                        set_is_loading_older_logs(true);
                        setTimeout(() => {
                            loadOlderLogs().then((can_load_older_logs) => {
                                set_can_load_older_logs(can_load_older_logs);
                                set_is_loading_older_logs(false);
                            });
                        }, 1000);
                    } }
                    leftSection={
                        is_loading_older_logs
                        ? <Loader size={ 14 }
                                  color={ "violet.3" }
                                  m={ 0 }
                        />
                        : <IconReload size={ 14 } />
                    }
                    size={ "xs" }
                    mb={ "xs" }
            >
                {
                    is_loading_older_logs
                    ? "Loading older logs ..."
                    : "Load older logs"
                }
            </Button>
            {
                logs.map((log, index) => <LogLine log={ log }
                                                  key={ index }
                />)
            }
        </div>
    );
};