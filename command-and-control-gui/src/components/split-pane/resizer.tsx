import React, {CSSProperties, FC} from 'react';

export const RESIZER_DEFAULT_CLASSNAME = 'Resizer';

interface ResizerProps {
    className: string;
    onClick?: (event: React.MouseEvent<HTMLSpanElement, MouseEvent>) => void;
    onDoubleClick?: (event: React.MouseEvent<HTMLSpanElement, MouseEvent>) => void;
    onMouseDown: (event: React.MouseEvent<HTMLSpanElement, MouseEvent>) => void;
    onTouchStart: (event: React.TouchEvent<HTMLSpanElement>) => void;
    onTouchEnd: (event: React.TouchEvent<HTMLSpanElement>) => void;
    split?: 'vertical' | 'horizontal';
    style?: CSSProperties;
    resizerClassName?: string;
}

const Resizer: FC<ResizerProps> = ({
                                       className,
                                       onClick,
                                       onDoubleClick,
                                       onMouseDown,
                                       onTouchStart,
                                       onTouchEnd,
                                       resizerClassName = RESIZER_DEFAULT_CLASSNAME,
                                       split,
                                       style,
                                   }) => {
    const classes = [resizerClassName, split, className].filter(Boolean).join(' ');

    return (
        <span
            role="presentation"
            className={classes}
            style={style}
            onMouseDown={(event) => onMouseDown(event)}
            onTouchStart={(event) => {
                event.preventDefault();
                onTouchStart(event);
            }}
            onTouchEnd={(event) => {
                event.preventDefault();
                onTouchEnd(event);
            }}
            onClick={(event) => {
                if (onClick) {
                    event.preventDefault();
                    onClick(event);
                }
            }}
            onDoubleClick={(event) => {
                if (onDoubleClick) {
                    event.preventDefault();
                    onDoubleClick(event);
                }
            }}
        />
    );
};

export default Resizer;
