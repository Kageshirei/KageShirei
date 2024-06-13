import { PostProcessHistory } from "@/components/post-process-command/history";
import { PostProcessSessions } from "@/components/post-process-command/sessions";
import {
    INTERNAL_HANDLES_LOOKUP,
    NATIVE_COMMANDS,
    NativeHandler,
    TERMINAL_EMULATOR_INTERNAL_HANDLES,
} from "@/components/terminal/native-commands";
import { TerminalInputLine } from "@/components/terminal/terminal-input-line";
import { TerminalOpenerSection } from "@/components/terminal/terminal-opener-section";
import { AuthenticationCtx } from "@/context/authentication";
import Ansi from "ansi-to-react";
import {
    all,
    isObject,
} from "radash";
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
    session_id: string | null;
    dropTerminalHandle: (hostname: string) => void;
    addTerminalHandle: (hostname: string, cwd: string, id: string) => void;
    style?: CSSProperties;
}

export const Terminal: FC<TerminalProps> = ({
    cwd,
    username,
    hostname,
    style,
    dropTerminalHandle,
    addTerminalHandle,
    session_id,
}) => {
    const [ requires_input_line_append, set_requires_input_line_append ] = useState(true);
    const [ terminal_fragments, set_terminal_fragments ] = useState<JSX.Element[]>([]);
    const [ terminal_history, set_terminal_history ] = useState<string[]>([]);
    const [ history_index, set_history_index ] = useState<number | null>(null);

    const handle_terminal_keydown = useCallback(
        async (e: KeyboardEvent<HTMLSpanElement>) => {
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
                    const response = await fetch(`http://${ AuthenticationCtx.host }/terminal`, {
                        method:  "POST",
                        headers: {
                            "Content-Type":  "application/json",
                            "Authorization": `Bearer ${ AuthenticationCtx.bearer }`,
                        },
                        body:    JSON.stringify({
                            command,
                            session_id,
                        }),
                    });

                    let json = await response.json();

                    console.log(json);

                    // handle frontend commands
                    if (
                        TERMINAL_EMULATOR_INTERNAL_HANDLES.some(v => json.response.startsWith(v))
                    ) {
                        const command = TERMINAL_EMULATOR_INTERNAL_HANDLES.filter(v => json.response.startsWith(v))[0];
                        let internal_call: NativeHandler = INTERNAL_HANDLES_LOOKUP[command];

                        // remove the internal command from the response, this allows the command to be parsed as a
                        // JSON if it contains other data
                        json.response = json.response.replace(command, "");

                        if (json.response.length > 0) {
                            try {
                                json.response = JSON.parse(json.response);
                            }
                            catch (e) {
                                // pass
                            }
                        }

                        console.log("json.response", json.response);

                        // handle the response and call the handlers recursively
                        if (Array.isArray(json.response)) {
                            await all(
                                json.response.map(async (elem: any) => {
                                    await NATIVE_COMMANDS[internal_call].handler({
                                        args:     elem.args ? elem.args : JSON.parse(elem.command),
                                        cwd:      elem.cwd ? elem.cwd : cwd,
                                        username: elem.username ? elem.username : username,
                                        hostname: elem.hostname ? elem.hostname : hostname,
                                        set_cwd:  () => {},
                                        set_terminal_fragments,
                                        terminal_fragments,
                                        hooks:    {
                                            dropTerminalHandle,
                                            addTerminalHandle,
                                        },
                                    });
                                }),
                            );
                        }
                        // one object only, pass the required arguments one time only
                        else if (isObject(json.response)) {
                            await NATIVE_COMMANDS[internal_call].handler({
                                args:     json.response.args ? json.response.args : JSON.parse(json.command),
                                cwd:      json.response.cwd ? json.response.cwd : cwd,
                                username: json.response.username ? json.response.username : username,
                                hostname: json.response.hostname ? json.response.hostname : hostname,
                                set_cwd:  () => {},
                                set_terminal_fragments,
                                terminal_fragments,
                                hooks:    {
                                    dropTerminalHandle,
                                    addTerminalHandle,
                                },
                            });
                        }
                        // no data, just call the handler with the parent arguments
                        else {
                            await NATIVE_COMMANDS[internal_call].handler({
                                args:    JSON.parse(json.command),
                                cwd,
                                username,
                                hostname,
                                set_cwd: () => {},
                                set_terminal_fragments,
                                terminal_fragments,
                                hooks:   {
                                    dropTerminalHandle,
                                    addTerminalHandle,
                                },
                            });
                        }
                    }
                    else {
                        try {
                            json = JSON.parse(json.response);
                            console.log("parsed json", json);
                        }
                        catch (e) {
                            // pass
                        }

                        // post parsing is required, so we need to handle the response manually and based on its type
                        // parse the json in the "data" field
                        if ("type" in json) {
                            switch (json.type) {
                                case "sessions":
                                    set_terminal_fragments(old => [
                                        ...old,
                                        <div key={ `${ hostname }-out-${ old.length + 1 }` }
                                             className="break-all whitespace-pre-wrap"
                                        >
                                            <PostProcessSessions sessions={ json.data } />
                                        </div>,
                                    ]);
                                    break;
                                case "history":
                                    set_terminal_fragments(old => [
                                        ...old,
                                        <div key={ `${ hostname }-out-${ old.length + 1 }` }
                                             className="break-all whitespace-pre-wrap"
                                        >
                                            <PostProcessHistory history={ json.data } />
                                        </div>,
                                    ]);
                                    break;
                                default:
                                    set_terminal_fragments(old => [
                                        ...old,
                                        <div key={ `${ hostname }-out-${ old.length + 1 }` }
                                             className="break-all whitespace-pre-wrap"
                                        >
                                            Response requires post-parsing, but no handler was found.
                                            <br />
                                            <br />
                                            <Ansi>
                                                { JSON.stringify(json) }
                                            </Ansi>
                                        </div>,
                                    ]);
                                    break;
                            }
                        }
                        else {
                            set_terminal_fragments(old => [
                                ...old,
                                <div key={ `${ hostname }-out-${ old.length + 1 }` }
                                     className="break-all whitespace-pre-wrap"
                                >
                                    <Ansi>
                                        { json.response }
                                    </Ansi>
                                </div>,
                            ]);
                        }
                    }

                    set_requires_input_line_append(true);

                    // focus on the input line
                    setTimeout(() => {
                        ([ ...document.querySelectorAll(`#${ hostname }-terminal-input-line`) ].at(-1) as HTMLSpanElement | undefined)?.focus();
                    }, 50);
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
        <div className="w-full p-4 bg-zinc-900 mt-2 rounded font-mono items-center relative min-h-[inherit]
        max-h-[inherit] h-full overflow-x-hidden overflow-y-auto pr-2 text-sm"
             style={ style }
        >
            {
                terminal_fragments.map(v => v)
            }
        </div>
    );
};
