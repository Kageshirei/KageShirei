use alloc::sync::Arc;
use rs2_runtime::Runtime;

#[cfg(feature = "std-runtime")]
use mod_std_runtime::StdRuntime;

#[cfg(feature = "nostd-nt-runtime")]
use mod_nostd_nt_runtime::NoStdNtRuntime;

pub fn initialize_runtime() -> Arc<impl Runtime> {
    #[cfg(feature = "std-runtime")]
    {
        Arc::new(StdRuntime::new(4))
    }

    #[cfg(feature = "nostd-nt-runtime")]
    {
        Arc::new(NoStdNtRuntime::new(4))
    }
}
