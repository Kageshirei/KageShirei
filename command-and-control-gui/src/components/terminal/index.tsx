import {FC, KeyboardEvent, useCallback, useEffect, useState} from "react";
import {TerminalOpenerSection} from "@/components/terminal/terminal-opener-section";
import {TerminalInputLine} from "@/components/terminal/terminal-input-line";

interface TerminalProps {
    hostname: string;
    username: string;
    cwd: string;
}

export const Terminal: FC<TerminalProps> = (
    {
        cwd,
        username,
        hostname,
    }
) => {
    const [terminal_fragments, set_terminal_fragments] = useState([
        <TerminalOpenerSection key={"none"} username={username} hostname={hostname} cwd={cwd}/>,
    ])

    const handle_terminal_keydown = useCallback((e: KeyboardEvent<HTMLSpanElement>) => {
        // handle the enter command
        if (e.key === "Enter") {
            e.preventDefault();
            e.stopPropagation();

            const command = e.currentTarget.innerText.trim();

            switch (command) {
                case "clear":
                    set_terminal_fragments([
                        <TerminalOpenerSection key={"none"} username={username} hostname={hostname} cwd={cwd}/>,
                        <TerminalInputLine key={"input"} handle_terminal_keydown={handle_terminal_keydown}/>
                    ])
                    return;
                case "help":
                    set_terminal_fragments((old) => [
                        ...old,
                        <div key={"help"}>
                            <ul>
                                <li>clear - clear the terminal</li>
                                <li>help - show this help message</li>
                            </ul>
                        </div>,
                        <TerminalInputLine key={"input"} handle_terminal_keydown={handle_terminal_keydown}/>
                    ])
                    return;
                default:
                    set_terminal_fragments(old => [
                        ...old,
                        <span key={"command"}>{command}</span>,
                        <TerminalInputLine key={"input"} handle_terminal_keydown={handle_terminal_keydown}/>
                    ])
            }
        }
    }, [cwd, hostname, terminal_fragments, username])

    useEffect(() => {
        set_terminal_fragments(old => [
            ...old,
            <TerminalInputLine key={"input"} handle_terminal_keydown={handle_terminal_keydown}/>
        ])
    }, []);

    return (
        <div className="w-full px-4 py-4 bg-zinc-900 mt-2 rounded font-mono items-center relative">
            {
                terminal_fragments.map(v => v)
            }
        </div>
    )
}