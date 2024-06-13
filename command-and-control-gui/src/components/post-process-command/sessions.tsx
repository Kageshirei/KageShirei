import {getTextualIntegrityLevel} from "@/helpers/textual-integrity-level";
import {SessionRecord} from "@/interfaces/session";
import {Table, TableScrollContainer, TableTbody, TableTd, TableTh, TableThead, TableTr,} from "@mantine/core";
import {FC, useMemo,} from "react";

interface PostParseHistoryProps {
    sessions: SessionRecord[];
}

export const PostProcessSessions: FC<PostParseHistoryProps> = ({sessions}) => {
    return (
        <TableScrollContainer minWidth={700}>
            <Table withColumnBorders
                   withRowBorders
                   my={"sm"}
            >
                <TableThead>
                    <TableTr>
                        <TableTh>Hostname</TableTh>
                        <TableTh>Domain\user</TableTh>
                        <TableTh>IP</TableTh>
                        <TableTh>OS</TableTh>
                        <TableTh>Integrity level</TableTh>
                    </TableTr>
                </TableThead>
                <TableTbody>
                    {
                        sessions.map((session, index) => (
                            <SessionLine
                                session_line={session}
                                key={session.id}
                            />
                        ))
                    }
                </TableTbody>
            </Table>
        </TableScrollContainer>
    );
};

interface HistoryLineProps {
    session_line: SessionRecord;
}

const SessionLine: FC<HistoryLineProps> = ({session_line}) => {
    const integrity_level = useMemo(() => getTextualIntegrityLevel(session_line.integrity_level), [session_line.integrity_level]);

    return (
        <TableTr>
            <TableTd className="font-semibold break-keep whitespace-nowrap">
                {session_line.hostname}
            </TableTd>
            <TableTd className="break-keep whitespace-nowrap">
                {session_line.domain}\{session_line.username}
            </TableTd>
            <TableTd className="break-keep whitespace-nowrap">
                {session_line.ip}
            </TableTd>
            <TableTd className="break-keep whitespace-nowrap">
                {session_line.operative_system.toLowerCase()}
            </TableTd>
            <TableTd className="break-all whitespace-pre-wrap">
                {integrity_level}
            </TableTd>
        </TableTr>
    );
};