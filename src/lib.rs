pub mod go;
pub mod python;
pub mod rust;
pub mod typescript;

use std::{fmt::Display, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tree_sitter::{LanguageError, QueryError};

const FUNC_NAME_CAPTURE: &str = "func.name";

/// The identifier of a function in the form of an "expected" autometrics label.
///
/// This label is given as a best effort most of the time, as some languages
/// cannot provide statically the exact information that is going to be produced
/// by Autometrics.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ExpectedAmLabel {
    /// The location of the definition of the function.
    pub module: String,
    /// The name of the function.
    pub function: String,
}

impl Display for ExpectedAmLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "module: {}, function: {}", self.module, self.function)
    }
}

/// Trait to implement to claim "Language support" for am_list.
///
/// This means we can both list all autometricized functions in a project, and
/// all functions defined without distinction in a project.
pub trait ListAmFunctions {
    /// List all the autometricized functions under the given project.
    fn list_autometrics_functions(&mut self, project_root: &Path) -> Result<Vec<ExpectedAmLabel>>;
    /// List all the functions defined in the given project.
    fn list_all_functions(&mut self, project_root: &Path) -> Result<Vec<ExpectedAmLabel>>;
}

pub type Result<T> = std::result::Result<T, AmlError>;

#[derive(Debug, Error)]
pub enum AmlError {
    /// Issue when trying to create a Tree-sitter parser.
    #[error("Issue creating the TreeSitter parser")]
    CreateParser(#[from] LanguageError),
    /// Issue when trying to create a Tree-sitter query.
    #[error("Issue creating the TreeSitter query")]
    CreateQuery(#[from] QueryError),
    /// Issue when the query is expected to have the given named capture.
    #[error("The query is missing an expected named capture: {0}")]
    MissingNamedCapture(String),
    /// Issue when parsing source code.
    #[error("Parsing error")]
    Parsing,
    /// Issue when trying to convert an extract of source code to a unicode
    /// String.
    #[error("Invalid text in source")]
    InvalidText,
}
