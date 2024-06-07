import {FC} from "react";

interface TerminalOpenerSectionProps {
    username: string,
    hostname: string,
    cwd: string,
}

export const TerminalOpenerSection: FC<TerminalOpenerSectionProps> = (
    {
        cwd,
        username,
        hostname
    }
) => {
    return (
        <>
            <span className="text-[#A6E22E] font-semibold text-base">
                {username}@{hostname}
            </span>
            :
            <span className="text-[#66D9EF] font-semibold text-base">
                {cwd}
            </span>
            $&nbsp;
        </>
    )
}