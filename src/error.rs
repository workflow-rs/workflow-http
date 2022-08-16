use std::{cell::{BorrowError, BorrowMutError}, path::PathBuf};
use serde_json::Error as SerdeError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("File not supported: {0}")]
    FileNotSupported(PathBuf),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serde error: {0}")]
    SerdeError(#[from] SerdeError),

    #[error("Borrow error: {0:?}")]
    BorrowError(#[from] BorrowError),
    
    #[error("Borrow mut error: {0:?}")]
    BorrowMutError(#[from] BorrowMutError),
}
