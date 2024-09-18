pub(crate) mod duration;

// helper macro to conditionally log trace messages
pub(crate) mod macros {
    macro_rules! _trace {
        ($($arg:tt)*) => {
            #[cfg(feature = "trace")]
            tracing::trace!($($arg)*);
        };
    }

    macro_rules! _debug {
        ($($arg:tt)*) => {
            #[cfg(feature = "trace")]
            tracing::debug!($($arg)*);
        };
    }

    macro_rules! _info {
        ($($arg:tt)*) => {
            #[cfg(feature = "trace")]
            tracing::info!($($arg)*);
        };
    }

    macro_rules! _warn {
        ($($arg:tt)*) => {
            #[cfg(feature = "trace")]
            tracing::warn!($($arg)*);
        };
    }

    pub(crate) use _trace;
}

pub fn canonicalize(path: impl AsRef<std::path::Path>) -> std::io::Result<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        dunce::canonicalize(path).and_then(|p| {
            let p = p.display().to_string();
            let prefixed = format!("/{}", p.replace("\\", "/"));
            Ok(prefixed.into())
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::fs::canonicalize(path)
    }
}
