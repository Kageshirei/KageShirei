import React, {CSSProperties, forwardRef, ReactNode} from 'react';

interface PaneProps {
    className: string;
    children: ReactNode;
    size?: string | number;
    split?: 'vertical' | 'horizontal';
    style?: CSSProperties;
    eleRef?: React.Ref<HTMLDivElement>;
}

const Pane = forwardRef<HTMLDivElement, PaneProps>((props, ref) => {
    const {children, className, split, style: styleProps, size, eleRef} = props;

    const classes = ['Pane', split, className].filter(Boolean).join(' ');

    let style: CSSProperties = {
        flex: 1,
        position: 'relative',
        outline: 'none',
    };

    if (size !== undefined) {
        if (split === 'vertical') {
            style.width = size;
        } else {
            style.height = size;
            style.display = 'flex';
        }
        style.flex = 'none';
    }

    style = {...style, ...styleProps};

    return (
        <div ref={eleRef || ref} className={classes} style={style}>
            {children}
        </div>
    );
});

Pane.displayName = 'Pane';

export default Pane;
