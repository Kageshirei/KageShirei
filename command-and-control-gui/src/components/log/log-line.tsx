import { dayjs } from "@/helpers/dayjs";
import { ILog } from "@/interfaces/log";
import {
    FC,
    useMemo,
} from "react";

export const LogLine: FC<{
    log: ILog
}> = ({ log }) => {
    const timestamp = useMemo(() => {
        return dayjs(log.created_at).format("MMM DD YYYY HH:mm:ss Z");
    }, [ log.created_at ]);

    const level_color = useMemo(() => {
        switch (log.level) {
            case "INFO":
                return "text-emerald-600";
            case "WARN":
                return "text-yellow-500";
            case "ERROR":
                return "text-red-500";
            case "DEBUG":
                return "text-blue-500";
            case "TRACE":
                return "text-gray-500";
        }
    }, [ log.level ]);

    return (
        <div className="flex items-center">
            <div className="text-gray-500">
                [{ timestamp }]{ " " }
            </div>
            <div className="flex flex-nowrap">
                {
                    Object.entries(log.extra).map(([ key, value ], index) => (
                        <div key={ key }
                             className={ "flex items-center" }
                        >
                            [
                            <div className="text-cyan-500">{ key }</div>
                            : { value }]{ " " }
                        </div>
                    ))
                }
            </div>
            <div className={ level_color }>
                { log.level }{ " " }
            </div>
            <div className="font-semibold">
                { log.title }:{ " " }
            </div>
            <div>
                { log.message }
            </div>
        </div>
    );
};