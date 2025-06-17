use std::{error::Error as StdError, panic::Location};

#[derive(Copy, Clone, Debug, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    Other,
    BadInstr,
    Io,

    #[doc(hidden)]
    __Uncategorized,
}

impl ErrorKind {
    pub const fn unrecognized(&self) -> bool {
        matches!(self, Self::Other | Self::__Uncategorized)
    }
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other => f.write_str("other"),
            Self::BadInstr => f.write_str("invalid instruction"),
            Self::Io => f.write_str("I/O operation error"),
            Self::__Uncategorized => f.write_str(""),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    payload: Option<Box<dyn StdError + Send + Sync + 'static>>,
    source_file: Option<String>,
    #[cfg(any(
        feature = "error-track-caller",
        all(feature = "debug-error-track-caller", debug_assertions)
    ))]
    #[allow(dead_code)]
    // This is intentionally only exposed via the `Debug` derive, as it's not part of the public API, but can be used by debug representation to understand partial support
    new_loc: &'static Location<'static>,
}

impl From<ErrorKind> for Error {
    #[inline]
    fn from(value: ErrorKind) -> Self {
        Self::constructor_impl(value, None, None)
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(value: std::io::Error) -> Self {
        Self::constructor_impl(ErrorKind::Io, Some(Box::new(value)), None)
    }
}

impl Error {
    #[cfg_attr(
        any(
            feature = "error-track-caller",
            all(feature = "debug-error-track-caller", debug_assertions)
        ),
        track_caller
    )]
    #[inline(always)]
    fn constructor_impl(
        kind: ErrorKind,
        payload: Option<Box<dyn StdError + Send + Sync>>,
        source_file: Option<String>,
    ) -> Self {
        Self {
            kind,
            payload,
            source_file,
            #[cfg(any(
                feature = "error-track-caller",
                all(feature = "debug-error-track-caller", debug_assertions)
            ))]
            new_loc: Location::caller(),
        }
    }

    #[cfg_attr(
        any(
            feature = "error-track-caller",
            all(feature = "debug-error-track-caller", debug_assertions)
        ),
        track_caller
    )]
    #[inline]
    pub fn new(
        kind: ErrorKind,
        payload: impl Into<Box<dyn StdError + Send + Sync + 'static>>,
    ) -> Self {
        Self::constructor_impl(kind, Some(payload.into()), None)
    }

    #[cfg_attr(
        any(
            feature = "error-track-caller",
            all(feature = "debug-error-track-caller", debug_assertions)
        ),
        track_caller
    )]
    #[inline]
    pub fn uncategorized(payload: impl Into<Box<dyn StdError + Send + Sync + 'static>>) -> Self {
        Self::constructor_impl(ErrorKind::__Uncategorized, Some(payload.into()), None)
    }

    #[cfg_attr(
        any(
            feature = "error-track-caller",
            all(feature = "debug-error-track-caller", debug_assertions)
        ),
        track_caller
    )]
    #[inline]
    pub fn new_with_source(
        kind: ErrorKind,
        payload: impl Into<Box<dyn StdError + Send + Sync + 'static>>,
        source: impl ToString,
    ) -> Self {
        Self::constructor_impl(kind, Some(payload.into()), Some(source.to_string()))
    }

    #[cfg_attr(
        any(
            feature = "error-track-caller",
            all(feature = "debug-error-track-caller", debug_assertions)
        ),
        track_caller
    )]
    #[inline]
    pub fn uncategorized_with_source(
        payload: impl Into<Box<dyn StdError + Send + Sync + 'static>>,
        source: impl ToString,
    ) -> Self {
        Self::constructor_impl(
            ErrorKind::__Uncategorized,
            Some(payload.into()),
            Some(source.to_string()),
        )
    }

    #[inline(always)]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    #[inline]
    pub fn payload(&self) -> Option<&(dyn StdError + Send + Sync + 'static)> {
        self.payload.as_deref()
    }

    #[inline]
    pub fn source_file(&self) -> Option<&str> {
        self.source_file.as_deref()
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(file) = self.source_file() {
            f.write_fmt(format_args!("[{file}] "))?;
        }

        self.kind.fmt(f)?;
        if let Some(payload) = self.payload() {
            if !matches!(self.kind, ErrorKind::__Uncategorized) {
                f.write_str(": ")?;
            }
            core::fmt::Display::fmt(payload, f)
        } else if matches!(self.kind, ErrorKind::__Uncategorized) {
            f.write_str("(uncategorized error)")
        } else {
            Ok(())
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.payload().map(|v| v as &_)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
