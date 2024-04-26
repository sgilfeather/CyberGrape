use std::{error::Error, fmt::Display, sync::mpsc};

#[derive(Debug)]
pub enum GrapeGuiError {
    FmtError(std::fmt::Error),
    IOError(std::io::Error),
    MPSCSendError,
    MPSCRecvError(mpsc::RecvError),
    MPSCTryRecvError(mpsc::TryRecvError),
    JoinError,
} 

impl Display for GrapeGuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Error for GrapeGuiError {}

impl From<std::fmt::Error> for GrapeGuiError {
    fn from(value: std::fmt::Error) -> Self {
        Self::FmtError(value)
    }
}

impl From<std::io::Error> for GrapeGuiError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl <T> From<mpsc::SendError<T>> for GrapeGuiError {
    fn from(_: mpsc::SendError<T>) -> Self {
        Self::MPSCSendError
    }
}

impl <T> From<mpsc::TrySendError<T>> for GrapeGuiError {
    fn from(_: mpsc::TrySendError<T>) -> Self {
        Self::MPSCSendError
    }
}

impl From<mpsc::RecvError> for GrapeGuiError {
    fn from(value: mpsc::RecvError) -> Self {
        Self::MPSCRecvError(value)
    }
}

impl From<mpsc::TryRecvError> for GrapeGuiError {
    fn from(value: mpsc::TryRecvError) -> Self {
        Self::MPSCTryRecvError(value)
    }
}
