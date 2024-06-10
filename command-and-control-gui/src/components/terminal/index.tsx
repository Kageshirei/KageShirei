import {
    NATIVE_COMMANDS,
    NativeHandler,
} from "@/components/terminal/native-commands";
import { TerminalInputLine } from "@/components/terminal/terminal-input-line";
import { TerminalOpenerSection } from "@/components/terminal/terminal-opener-section";
import Ansi from "ansi-to-react";
import {
    CSSProperties,
    FC,
    JSX,
    KeyboardEvent,
    useCallback,
    useEffect,
    useState,
} from "react";

interface TerminalProps {
    hostname: string;
    username: string;
    cwd: string;
    dropTerminalHandle: (hostname: string) => void;
    style?: CSSProperties;
}

export const Terminal: FC<TerminalProps> = ({
    cwd,
    username,
    hostname,
    style,
    dropTerminalHandle,
}) => {
    const [ requires_input_line_append, set_requires_input_line_append ] = useState(true);
    const [ terminal_fragments, set_terminal_fragments ] = useState<JSX.Element[]>([]);
    const [ terminal_history, set_terminal_history ] = useState<string[]>([]);
    const [ history_index, set_history_index ] = useState<number | null>(null);

    const handle_terminal_keydown = useCallback(
        (e: KeyboardEvent<HTMLSpanElement>) => {
            if (e.key === "Enter") {
                e.preventDefault();
                e.stopPropagation();

                const command = e.currentTarget.innerText.trim();
                if (command) {
                    // Add the command to the history
                    set_terminal_history(old => {
                        const newHistory = [
                            ...old,
                            command,
                        ];
                        set_history_index(newHistory.length);  // This will set to one past the last item

                        return newHistory;
                    });

                    // send the command to the backend
                    import("@/context/authentication").then(async ({ AuthenticationCtx }) => {
                        const response = await fetch(`http://${ AuthenticationCtx.host }/terminal`, {
                            method:  "POST",
                            headers: {
                                "Content-Type":  "application/json",
                                "Authorization": `Bearer ${ AuthenticationCtx.bearer }`,
                            },
                            body:    JSON.stringify({
                                command,
                                session_id: null,
                            }),
                        });

                        const json = await response.json();

                        console.log(json);

                        // handle frontend commands
                        if ([
                            "__TERMINAL_EMULATOR_INTERNAL_HANDLE_CLEAR__",
                            "__TERMINAL_EMULATOR_INTERNAL_HANDLE_EXIT__",
                        ].includes(json.response)) {
                            let internal_call: NativeHandler | null = null;

                            switch (json.response) {
                                case "__TERMINAL_EMULATOR_INTERNAL_HANDLE_CLEAR__":
                                    internal_call = "clear";
                                    break;
                                case "__TERMINAL_EMULATOR_INTERNAL_HANDLE_EXIT__":
                                    internal_call = "exit";
                                    break;
                            }

                            if (internal_call) {
                                await NATIVE_COMMANDS[internal_call].handler({
                                    args:    JSON.parse(json.command),
                                    cwd,
                                    username,
                                    hostname,
                                    set_cwd: () => { },
                                    set_terminal_fragments,
                                    terminal_fragments,
                                    hooks:   {
                                        dropTerminalHandle,
                                    },
                                });
                            }
                        }
                        else {
                            set_terminal_fragments(old => [
                                ...old,
                                <pre key={ `${ hostname }-out-${ old.length + 1 }` }
                                     className="break-all"
                                >
                                    <Ansi>
                                        { json.response }
                                    </Ansi>
                                </pre>,
                            ]);
                        }

                        set_requires_input_line_append(true);

                        // focus on the input line
                        setTimeout(() => {
                            ([ ...document.querySelectorAll(`#${ hostname }-terminal-input-line`) ].at(-1) as HTMLSpanElement | undefined)?.focus();
                        }, 50);
                    });
                }
            }
            // handle the up arrow key
            else if (e.key === "ArrowUp") {
                e.preventDefault();
                e.stopPropagation();

                if (terminal_history.length > 0) {
                    // must extract the target from the event to prevent a stale closure
                    const target = e.currentTarget;

                    set_history_index(old => {
                        const new_index = old === null ? terminal_history.length - 1 : Math.max(
                            old - 1,
                            0,
                        );
                        target.innerText = terminal_history[new_index] || "";

                        return new_index;
                    });
                }
            }
            else if (e.key === "ArrowDown") {
                e.preventDefault();
                e.stopPropagation();

                if (terminal_history.length > 0 && history_index !== null) {
                    // must extract the target from the event to prevent a stale closure
                    const target = e.currentTarget;

                    set_history_index(old => {
                        const new_index = old === null ? terminal_history.length : Math.min(
                            old + 1,
                            terminal_history.length,
                        );
                        target.innerText = terminal_history[new_index] || "";

                        return new_index;
                    });
                }
            }
        },
        [
            cwd,
            history_index,
            hostname,
            terminal_fragments,
            terminal_history,
            username,
        ],
    );

    useEffect(
        () => {
            if (requires_input_line_append) {
                set_requires_input_line_append(false);

                set_terminal_fragments(old => {
                    // if the last element is an input line, don't append another
                    const last = old.at(-1);
                    if (last && last.type === TerminalInputLine) {
                        return old;
                    }

                    return [
                        ...old,
                        <TerminalOpenerSection key={ `${ hostname }-tos-${ old.length + 1 }` }
                                               username={ username }
                                               hostname={ hostname }
                                               cwd={ cwd }
                        />,
                        <TerminalInputLine key={ `${ hostname }-til-${ old.length + 2 }` }
                                           handle_terminal_keydown={ handle_terminal_keydown }
                                           hostname={ hostname }
                        />,
                    ];
                });
            }
        },
        [
            cwd,
            handle_terminal_keydown,
            hostname,
            requires_input_line_append,
            username,
        ],
    );

    return (
        <div className="w-full px-4 py-4 bg-zinc-900 mt-2 rounded font-mono items-center relative min-h-[inherit]
        max-h-[inherit] h-full overflow-x-hidden overflow-y-auto pr-2 text-sm"
             style={ style }
        >
            {
                terminal_fragments.map(v => v)
            }
        </div>
    );
};
