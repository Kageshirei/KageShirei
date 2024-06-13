import {FC} from "react";
import {HistoryRecord} from "@/interfaces/history";
import {Table, TableScrollContainer, TableTbody, TableTd, TableTh, TableThead, TableTr} from "@mantine/core";
import {dayjs} from "@/helpers/dayjs";

interface PostParseHistoryProps {
    history: HistoryRecord[];
}

export const PostParseHistory: FC<PostParseHistoryProps> = ({history}) => {
    return (
        <TableScrollContainer minWidth={700}>
            <Table withColumnBorders withRowBorders my={"sm"}>
                <TableThead>
                    <TableTr>
                        <TableTh>#</TableTh>
                        <TableTh>UID</TableTh>
                        <TableTh>Datetime</TableTh>
                        <TableTh>Command</TableTh>
                    </TableTr>
                </TableThead>
                <TableTbody>
                    {
                        history.map((history_line, index) => (
                            <HistoryLine
                                history_line={history_line}
                                index={index}
                                key={index}
                            />
                        ))
                    }
                </TableTbody>
            </Table>
        </TableScrollContainer>
    );
}

interface HistoryLineProps {
    history_line: HistoryRecord;
    index: number;
}

const HistoryLine: FC<HistoryLineProps> = ({history_line, index}) => {
    return (
        <TableTr>
            <TableTd className="font-semibold text-gray-500 break-keep whitespace-nowrap" align={"right"}>
                {index + 1}
            </TableTd>
            <TableTd className="break-keep whitespace-nowrap">
                {history_line.ran_by}
            </TableTd>
            <TableTd className="break-keep whitespace-nowrap">
                {
                    dayjs(history_line.created_at).format("MMM DD YYYY, HH:mm:ss")
                }
            </TableTd>
            <TableTd className="break-all whitespace-pre-wrap">
                {history_line.command}
            </TableTd>
        </TableTr>
    );
}