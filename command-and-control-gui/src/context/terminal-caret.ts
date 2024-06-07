import {proxy} from "valtio";
import {CSSProperties, Dispatch, SetStateAction} from "react";

class TerminalCaret {
    public set_style: Dispatch<SetStateAction<CSSProperties>> | undefined;
}

export const TerminalCaretCtx = proxy(new TerminalCaret)