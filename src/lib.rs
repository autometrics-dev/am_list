pub mod rust;

use std::{path::Path, fmt::Display};

use thiserror::Error;
use tree_sitter::{LanguageError, QueryError};

const FUNC_NAME_CAPTURE: &str = "func.name";

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpectedAmLabel {
    pub module: String,
    pub function: String,
}

impl Display for ExpectedAmLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "module: {}, function: {}", self.module, self.function)
    }
}

pub trait ListAmFunctions {
    fn list_autometrics_functions(&mut self, project_root: &Path) -> Result<Vec<ExpectedAmLabel>>;
}

pub type Result<T> = std::result::Result<T, AmlError>;

#[derive(Debug, Error)]
pub enum AmlError {
    #[error("Issue creating the TreeSitter parser")]
    CreateParser(#[from] LanguageError),
    #[error("Issue creating the TreeSitter query")]
    CreateQuery(#[from] QueryError),
    #[error("The query is missing a function name capture")]
    MissingFuncNameCapture,
    #[error("Parsing error")]
    Parsing,
    #[error("Invalid text in source")]
    InvalidText,
}
