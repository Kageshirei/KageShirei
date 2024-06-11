import { LogLine } from "@/components/log/log-line";
import { ILog } from "@/interfaces/log";
import { FC } from "react";

interface LogProps {
    logs: ILog[];
}

export const Log: FC<LogProps> = ({
    logs,
}) => {
    return (
        <div className="w-full p-4 bg-zinc-900 rounded font-mono relative h-full pr-6 text-sm max-h-full whitespace-pre">
            {
                logs.map((log, index) => <LogLine log={ log }
                                                  key={ index }
                />)
            }
        </div>
    );
};