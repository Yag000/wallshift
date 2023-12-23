use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WallshiftError {
    #[error("Parsing error")]
    Parsing {
        #[from]
        source: ParsingError,
    },
    #[error("File error")]
    File {
        #[from]
        source: FileError,
    },
    #[error("Exec error")]
    Exec {
        #[from]
        source: ExecError,
    },
}

#[derive(Error, Debug)]
pub struct ParsingError {
    pub message: String,
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl Into<ParsingError> for std::io::Error {
    fn into(self) -> ParsingError {
        ParsingError {
            message: format!("failed to parse: {}", self),
        }
    }
}

#[derive(Error, Debug)]
pub struct FileError {
    pub message: String,
}

impl Display for FileError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl Into<FileError> for Option<PathBuf> {
    fn into(self) -> FileError {
        FileError {
            message: format!("failed to get path from option: {:?}", self),
        }
    }
}

impl Into<FileError> for String {
    fn into(self) -> FileError {
        FileError {
            message: format!("failed to get path from string: {:?}", self),
        }
    }
}
impl Into<FileError> for &str {
    fn into(self) -> FileError {
        FileError {
            message: format!("failed to get path from string: {:?}", self),
        }
    }
}

#[derive(Error, Debug)]
pub struct ExecError {
    pub message: String,
}

impl Display for ExecError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}
