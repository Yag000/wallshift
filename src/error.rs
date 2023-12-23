use std::fmt::{Display, Formatter};

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

#[derive(Error, Debug)]
pub struct FileError {
    pub message: String,
}

impl Display for FileError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
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
