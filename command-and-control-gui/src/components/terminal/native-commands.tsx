import Ansi from "ansi-to-react";
import {
    Dispatch,
    JSX,
    SetStateAction,
} from "react";

interface CommandHandlerArguments {
    args: string[];
    cwd: string;
    username: string;
    hostname: string;
    set_cwd: (cwd: string) => void;
    set_terminal_fragments: Dispatch<SetStateAction<JSX.Element[]>>;
    terminal_fragments: JSX.Element[];
    hooks: {
        dropTerminalHandle: (hostname: string) => void;
    };
}

type CommandHandler = (args: CommandHandlerArguments) => void | Promise<void>;

interface CommandDefinition {
    description: string;
    handler: CommandHandler;
}

export type NativeHandler = "clear" | "exit";

export const NATIVE_COMMANDS: { [x in NativeHandler]: CommandDefinition } = {
    clear: {
        description: "Clear the terminal",
        handler: ({ set_terminal_fragments }) => {
            set_terminal_fragments([]);
        },
    },
    exit:  {
        description: "Exit the current terminal",
        handler:     ({
            hooks,
            hostname,
            set_terminal_fragments,
        }) => {
            // Using the hostname as the terminal handle avoids the possibility to exit the global terminal as it does
            // not follow the common hostname convention. This is a security measure to prevent the user from closing
            // the global terminal by accident. The global terminal is the terminal that is always open and is used to
            // run global commands. Even if the global terminal cannot be closed dropping it will break the line input,
            // this is the reason why the global terminal is not closed.
            if (hostname === "RS2") {
                // if the terminal is the global terminal, do not exit and show an error message
                set_terminal_fragments(old => [
                    ...old,
                    <pre key={ `${ hostname }-out-${ old.length + 1 }` }
                         className="break-all"
                    >
                        <Ansi>
                            &#x1B;[1m&#x1B;[31mError:&#x1B;[0m The global terminal cannot be closed.
                        </Ansi>
                    </pre>,
                ]);

                return;
            }

            hooks.dropTerminalHandle(hostname);
        },
    },
};