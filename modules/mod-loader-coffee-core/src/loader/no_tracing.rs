#[macro_export]
macro_rules! debug {
    // Name / target / parent.
    (name: $name:expr,target: $target:expr,parent: $parent:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Name / target.
    (name: $name:expr,target: $target:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Target / parent.
    (target: $target:expr,parent: $parent:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Name / parent.
    (name: $name:expr,parent: $parent:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Name.
    (name: $name:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, ? $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, % $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Target.
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, ? $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, % $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Parent.
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, ? $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, % $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, ? $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, % $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // ...
    ({ $($field:tt)+ }, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };
    ($($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (? $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (% $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    ($($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (? $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (% $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (? $($k:ident).+) => {
        // unimplemented as feature is disabled
    };
    (% $($k:ident).+) => {
        // unimplemented as feature is disabled
    };
    ($($k:ident).+) => {
        // unimplemented as feature is disabled
    };
    ($($arg:tt)+) => {
        // unimplemented as feature is disabled
    };
}

#[macro_export]
macro_rules! info {
    // Name / target / parent.
    (name: $name:expr,target: $target:expr,parent: $parent:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Name / target.
    (name: $name:expr,target: $target:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Target / parent.
    (target: $target:expr,parent: $parent:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Name / parent.
    (name: $name:expr,parent: $parent:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Name.
    (name: $name:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, ? $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, % $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Target.
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, ? $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, % $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Parent.
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, ? $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, % $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, ? $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, % $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // ...
    ({ $($field:tt)+ }, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };
    ($($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (? $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (% $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    ($($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (? $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (% $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (? $($k:ident).+) => {
        // unimplemented as feature is disabled
    };
    (% $($k:ident).+) => {
        // unimplemented as feature is disabled
    };
    ($($k:ident).+) => {
        // unimplemented as feature is disabled
    };
    ($($arg:tt)+) => {
        // unimplemented as feature is disabled
    };
}

#[macro_export]
macro_rules! warn {
    // Name / target / parent.
    (name: $name:expr,target: $target:expr,parent: $parent:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr,parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Name / target.
    (name: $name:expr,target: $target:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,target: $target:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Target / parent.
    (target: $target:expr,parent: $parent:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr,parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Name / parent.
    (name: $name:expr,parent: $parent:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, ? $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, % $($k:ident).+ $($field:tt)+) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr,parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Name.
    (name: $name:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, ? $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, % $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (name: $name:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Target.
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, ? $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, % $($k:ident).+ $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (target: $target:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // Parent.
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, ? $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, % $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, ? $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, % $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (parent: $parent:expr, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };

    // ...
    ({ $($field:tt)+ }, $($arg:tt)+) => {
        // unimplemented as feature is disabled
    };
    ($($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (? $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (% $($k:ident).+ = $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    ($($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (? $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (% $($k:ident).+, $($field:tt)*) => {
        // unimplemented as feature is disabled
    };
    (? $($k:ident).+) => {
        // unimplemented as feature is disabled
    };
    (% $($k:ident).+) => {
        // unimplemented as feature is disabled
    };
    ($($k:ident).+) => {
        // unimplemented as feature is disabled
    };
    ($($arg:tt)+) => {
        // unimplemented as feature is disabled
    };
}
