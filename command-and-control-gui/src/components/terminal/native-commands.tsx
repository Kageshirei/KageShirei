import {Dispatch, JSX, SetStateAction,} from "react";

interface CommandHandlerArguments {
    args: string[];
    cwd: string;
    username: string;
    hostname: string;
    set_cwd: (cwd: string) => void;
    set_terminal_fragments: Dispatch<SetStateAction<JSX.Element[]>>;
    terminal_fragments: JSX.Element[];
}

type CommandHandler = (args: CommandHandlerArguments) => void | Promise<void>;

interface CommandDefinition {
    description: string;
    handler: CommandHandler;
}

export type NativeHandler = "clear";

export const NATIVE_COMMANDS: { [x in NativeHandler]: CommandDefinition } = {
    clear: {
        description: "Clear the terminal",
        handler: ({set_terminal_fragments}) => {
            set_terminal_fragments([]);
        },
    },
};