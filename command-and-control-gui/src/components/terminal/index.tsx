import {NATIVE_COMMANDS} from "@/components/terminal/native-commands";
import {TerminalInputLine} from "@/components/terminal/terminal-input-line";
import {TerminalOpenerSection} from "@/components/terminal/terminal-opener-section";
import Ansi from "ansi-to-react";
import {CSSProperties, FC, KeyboardEvent, useCallback, useEffect, useState,} from "react";

interface TerminalProps {
    hostname: string;
    username: string;
    cwd: string;
    style?: CSSProperties;
}

export const Terminal: FC<TerminalProps> = (
    {
        cwd,
        username,
        hostname,
        style,
    },
) => {
    const [terminal_fragments, set_terminal_fragments] = useState([
        <TerminalOpenerSection key={0}
                               username={username}
                               hostname={hostname}
                               cwd={cwd}
        />,
    ]);

    const handle_terminal_keydown = useCallback(
        async (e: KeyboardEvent<HTMLSpanElement>) => {
            const command = e.currentTarget.innerText.trim();
            const {AuthenticationCtx} = await import("@/context/authentication");

            // handle the enter command
            if (e.key === "Enter") {
                e.preventDefault();
                e.stopPropagation();

                // if (NATIVE_COMMANDS[command]) {
                const response = await fetch(`http://${AuthenticationCtx.host}/terminal`, {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                        "Authorization": `Bearer ${AuthenticationCtx.bearer}`,
                    },
                    body: JSON.stringify({
                        command,
                        session_id: null,
                    }),
                });
                const json = await response.json();
                console.log(json);

                if (json.response === "__TERMINAL_EMULATOR_INTERNAL_HANDLE_CLEAR__") {
                    NATIVE_COMMANDS.clear.handler({
                        args: JSON.parse(json.command),
                        cwd,
                        username,
                        hostname,
                        set_cwd: () => {
                        },
                        set_terminal_fragments,
                        terminal_fragments,
                    });
                } else {
                    set_terminal_fragments(old => [
                        ...old,
                        <pre key={old.length + 1}
                             className="break-all"
                        >
                                <Ansi>
                                {json.response}
                                </Ansi>
                            </pre>,
                    ]);
                }

                // append a new input line
                set_terminal_fragments(old => [
                    ...old,
                    <TerminalOpenerSection key={old.length + 1}
                                           username={username}
                                           hostname={hostname}
                                           cwd={cwd}
                    />,
                    <TerminalInputLine key={old.length + 2}
                                       handle_terminal_keydown={handle_terminal_keydown}
                                       hostname={hostname}
                    />,
                ]);

                // focus on the input line
                setTimeout(() => {
                    ([...document.querySelectorAll(`#${hostname}-terminal-input-line`)].at(-1) as HTMLSpanElement | undefined)?.focus();
                }, 50);
            }
        },
        [
            cwd,
            hostname,
            username,
        ],
    );

    useEffect(() => {
        set_terminal_fragments(old => [
            ...old,
            <TerminalInputLine key={1}
                               handle_terminal_keydown={handle_terminal_keydown}
                               hostname={hostname}
            />,
        ]);
    }, []);

    return (
        <div className="w-full px-4 py-4 bg-zinc-900 mt-2 rounded font-mono items-center relative min-h-[inherit]
        max-h-[inherit] h-full overflow-x-hidden overflow-y-auto pr-2 text-sm"
             style={style}
        >
            {
                terminal_fragments.map(v => v)
            }
        </div>
    );
};