use cuid2::CuidConstructor;
use once_cell::sync::Lazy;

pub static CUID2: Lazy<CuidConstructor> = Lazy::new(|| {
    let mut cuid2 = CuidConstructor::new();
    cuid2.set_length(32);
    cuid2
});
