import React, {CSSProperties, ReactNode, useEffect, useRef, useState,} from "react";
import Pane from "./pane";
import Resizer, {RESIZER_DEFAULT_CLASSNAME} from "./resizer";

interface SplitPaneProps {
    allowResize?: boolean;
    children: ReactNode[];
    className?: string;
    primary?: "first" | "second";
    minSize?: string | number;
    maxSize?: string | number;
    defaultSize?: string | number;
    size?: string | number;
    split?: "vertical" | "horizontal";
    onDragStarted?: () => void;
    onDragFinished?: (newSize: number | string) => void;
    onChange?: (newSize: number | string) => void;
    onResizerClick?: () => void;
    onResizerDoubleClick?: () => void;
    style?: CSSProperties;
    resizerStyle?: CSSProperties;
    paneClassName?: string;
    pane1ClassName?: string;
    pane2ClassName?: string;
    paneStyle?: CSSProperties;
    pane1Style?: CSSProperties;
    pane2Style?: CSSProperties;
    resizerClassName?: string;
    step?: number;
}

const SplitPane: React.FC<SplitPaneProps> = (props) => {
    const {
        allowResize = true,
        children,
        className,
        primary = "first",
        minSize = 50,
        maxSize,
        defaultSize,
        size,
        split = "vertical",
        onDragStarted,
        onDragFinished,
        onChange,
        onResizerClick,
        onResizerDoubleClick,
        style: styleProps,
        resizerStyle,
        paneClassName,
        pane1ClassName,
        pane2ClassName,
        paneStyle,
        pane1Style: pane1StyleProps,
        pane2Style: pane2StyleProps,
        resizerClassName,
        step,
    } = props;

    const [active, setActive] = useState(false);
    const [position, setPosition] = useState<number | null>(null);
    const [draggedSize, setDraggedSize] = useState<number | string | undefined>(size ?? defaultSize);
    const [pane1Size, setPane1Size] = useState<number | string | undefined>(primary === "first"
        ? draggedSize
        : undefined);
    const [pane2Size, setPane2Size] = useState<number | string | undefined>(primary === "second"
        ? draggedSize
        : undefined);

    const splitPaneRef = useRef<HTMLDivElement>(null);
    const pane1Ref = useRef<HTMLDivElement>(null);
    const pane2Ref = useRef<HTMLDivElement>(null);

    const unFocus = (document: Document, window: Window) => {
        const selection = document.getSelection();
        if (selection) {
            selection.empty();
        } else {
            try {
                window.getSelection()?.removeAllRanges();
            } catch (e) {
            }
        }
    };

    const getDefaultSize = (
        defaultSize?: string | number,
        minSize?: string | number,
        maxSize?: string | number,
        draggedSize?: number | null,
    ) => {
        if (typeof draggedSize === "number") {
            const min = typeof minSize === "number" ? minSize : 0;
            const max = typeof maxSize === "number" && maxSize >= 0 ? maxSize : Infinity;
            return Math.max(min, Math.min(max, draggedSize));
        }
        if (defaultSize !== undefined) {
            return defaultSize;
        }
        return minSize;
    };

    const onMouseDown = (event: React.MouseEvent<HTMLDivElement>) => {
        const eventWithTouches = {
            ...event,
            touches: [
                {
                    clientX: event.clientX,
                    clientY: event.clientY,
                },
            ],
        };
        onTouchStart(eventWithTouches);
    };

    const onTouchStart = (event: any) => {
        if (allowResize) {
            unFocus(document, window);
            const pos = split === "vertical" ? event.touches[0].clientX : event.touches[0].clientY;

            if (typeof onDragStarted === "function") {
                onDragStarted();
            }

            setActive(true);
            setPosition(pos);
        }
    };

    const onMouseMove = (event: MouseEvent) => {
        const eventWithTouches = {
            ...event,
            touches: [
                {
                    clientX: event.clientX,
                    clientY: event.clientY,
                },
            ],
        };
        onTouchMove(eventWithTouches);
    };

    const onTouchMove = (event: any) => {
        if (allowResize && active) {
            unFocus(document, window);
            const isPrimaryFirst = primary === "first";
            const ref = isPrimaryFirst ? pane1Ref.current : pane2Ref.current;
            const ref2 = isPrimaryFirst ? pane2Ref.current : pane1Ref.current;

            if (ref) {
                const size = split === "vertical"
                    ? ref.getBoundingClientRect().width
                    : ref.getBoundingClientRect().height;
                const current = split === "vertical" ? event.touches[0].clientX : event.touches[0].clientY;
                let positionDelta = position! - current;
                if (step) {
                    if (Math.abs(positionDelta) < step) {
                        return;
                    }
                    positionDelta = Math.floor(positionDelta / step) * step;
                }
                let sizeDelta = isPrimaryFirst ? positionDelta : -positionDelta;

                const pane1Order = parseInt(window.getComputedStyle(ref).order);
                const pane2Order = parseInt(window.getComputedStyle(ref2!).order);
                if (pane1Order > pane2Order) {
                    sizeDelta = -sizeDelta;
                }

                let newSize = size - sizeDelta;
                let newPosition = position! - positionDelta;

                if (+newSize < +minSize) {
                    newSize = +minSize;
                } else if (typeof maxSize === "number" && newSize > maxSize) {
                    newSize = maxSize;
                } else {
                    setPosition(newPosition);
                }

                setDraggedSize(newSize);
                if (onChange) {
                    onChange(newSize);
                }
                if (isPrimaryFirst) {
                    setPane1Size(newSize);
                } else {
                    setPane2Size(newSize);
                }
            }
        }
    };

    const onMouseUp = () => {
        if (allowResize && active) {
            if (typeof onDragFinished === "function") {
                onDragFinished(draggedSize!);
            }
            setActive(false);
        }
    };

    useEffect(
        () => {
            document.addEventListener("mouseup", onMouseUp);
            document.addEventListener("mousemove", onMouseMove);
            document.addEventListener("touchmove", onTouchMove);

            return () => {
                document.removeEventListener("mouseup", onMouseUp);
                document.removeEventListener("mousemove", onMouseMove);
                document.removeEventListener("touchmove", onTouchMove);
            };
        },
        [
            active,
            position,
            draggedSize,
        ],
    );

    useEffect(
        () => {
            setDraggedSize(size ?? getDefaultSize(defaultSize, minSize, maxSize, null));
        },
        [
            size,
            defaultSize,
            minSize,
            maxSize,
        ],
    );

    const notNullChildren = React.Children.toArray(children).filter(c => c);

    const style: CSSProperties = {
        display: "flex",
        flex: 1,
        height: "100%",
        // position:         "absolute",
        outline: "none",
        // overflow:         "hidden",
        MozUserSelect: "text",
        WebkitUserSelect: "text",
        msUserSelect: "text",
        userSelect: "text",
        ...styleProps,
    };

    if (split === "vertical") {
        style.flexDirection = "row";
        style.left = 0;
        style.right = 0;
    } else {
        style.flexDirection = "column";
        style.bottom = 0;
        style.minHeight = "100%";
        style.top = 0;
        style.width = "100%";
    }

    const classes = [
        "SplitPane",
        className,
        split,
        allowResize ? "" : "disabled",
    ].filter(Boolean).join(" ");

    const pane1Style = {...paneStyle, ...pane1StyleProps};
    const pane2Style = {...paneStyle, ...pane2StyleProps};

    const pane1Classes = [
        "Pane1",
        paneClassName,
        pane1ClassName,
    ].join(" ");
    const pane2Classes = [
        "Pane2",
        paneClassName,
        pane2ClassName,
    ].join(" ");

    return (
        <div className={classes}
             ref={splitPaneRef}
             style={style}
        >
            <Pane
                className={pane1Classes}
                ref={pane1Ref}
                size={pane1Size}
                split={split}
                style={pane1Style}
            >
                {notNullChildren[0]}
            </Pane>
            <Resizer
                className={allowResize ? "" : "disabled"}
                onClick={onResizerClick}
                onDoubleClick={onResizerDoubleClick}
                onMouseDown={onMouseDown}
                onTouchStart={onTouchStart}
                onTouchEnd={onMouseUp}
                resizerClassName={resizerClassName
                    ? `${resizerClassName} ${RESIZER_DEFAULT_CLASSNAME}`
                    : RESIZER_DEFAULT_CLASSNAME}
                split={split}
                style={resizerStyle || {}}
            />
            <Pane
                className={pane2Classes}
                ref={pane2Ref}
                size={pane2Size}
                split={split}
                style={pane2Style}
            >
                {notNullChildren[1]}
            </Pane>
        </div>
    );
};

export default SplitPane;
