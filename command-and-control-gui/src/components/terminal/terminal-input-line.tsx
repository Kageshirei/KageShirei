import {
    createRef,
    FC,
    KeyboardEvent,
    useCallback,
    useEffect,
    useState,
} from "react";

interface TerminalInputLineProps {
    handle_terminal_keydown: (e: KeyboardEvent<HTMLSpanElement>) => void;
    hostname: string;
}

export const TerminalInputLine: FC<TerminalInputLineProps> = ({
    handle_terminal_keydown,
    hostname,
}) => {
    const [ is_editable, set_is_editable ] = useState(true);
    const ref = createRef<HTMLSpanElement>();

    const wrapped_terminal_keydown_handler = useCallback(
        (
            e: KeyboardEvent<HTMLSpanElement>,
        ) => {
            // disable the input line if the user presses enter
            if (e.key === "Enter") {
                set_is_editable(false);
            }

            if (e.key === "a" && e.ctrlKey) {
                // select all text in the current element
                e.preventDefault();
                const selection = window.getSelection();
                const range = document.createRange();
                range.selectNodeContents(e.currentTarget);
                selection?.removeAllRanges();
                selection?.addRange(range);
                return;
            }

            // call the user defined keydown handler
            handle_terminal_keydown(e);

            // if the user deletes all the text, add a zero width space to allow the user to continue typing
            if ((e.key === "Backspace" || e.key === "Delete")) {
                setTimeout(() => {
                    if (ref.current?.innerText.length === 0) {
                        ref.current.innerText = "\uFEFF";
                    }
                }, 50);
            }
        },
        [
            handle_terminal_keydown,
            ref,
        ],
    );

    useEffect(() => {
        if (ref.current) {
            ref.current.innerText = "\uFEFF";
        }
    }, []);

    return (
        <span ref={ ref }
              contentEditable={ is_editable }
              className="appearance-none font-mono outline-none border-none shadow-none
              before:content-['\feff'] min-w-full w-full h-3 break-all"
              onKeyDown={ wrapped_terminal_keydown_handler }
              id={ `${ hostname }-terminal-input-line` }
        />
    );
};