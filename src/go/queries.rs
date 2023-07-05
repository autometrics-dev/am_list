use crate::{AmlError, ExpectedAmLabel, Result, FUNC_NAME_CAPTURE};
use log::error;
use tree_sitter::{Parser, Query};
use tree_sitter_go::language;

const PACK_NAME_CAPTURE: &str = "pack.name";

fn new_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(language())?;
    Ok(parser)
}

#[derive(Debug)]
pub(super) struct AmQuery {
    query: Query,
    func_name_idx: u32,
    mod_name_idx: u32,
}

impl AmQuery {
    pub fn try_new() -> Result<Self> {
        let query = Query::new(
            language(),
            include_str!("../../runtime/queries/go/autometrics.scm"),
        )?;
        let func_name_idx = query
            .capture_index_for_name(FUNC_NAME_CAPTURE)
            .ok_or_else(|| AmlError::MissingNamedCapture(FUNC_NAME_CAPTURE.to_string()))?;
        let mod_name_idx = query
            .capture_index_for_name(PACK_NAME_CAPTURE)
            .ok_or_else(|| AmlError::MissingNamedCapture(PACK_NAME_CAPTURE.to_string()))?;

        Ok(Self {
            query,
            func_name_idx,
            mod_name_idx,
        })
    }

    pub fn list_function_names(&self, source: &str) -> Result<Vec<ExpectedAmLabel>> {
        let mut parser = new_parser()?;
        let parsed_source = parser.parse(source, None).ok_or(AmlError::Parsing)?;

        let mut cursor = tree_sitter::QueryCursor::new();
        cursor
            .matches(&self.query, parsed_source.root_node(), source.as_bytes())
            .filter_map(|capture| -> Option<Result<ExpectedAmLabel>> {
                let module = capture
                    .nodes_for_capture_index(self.mod_name_idx)
                    .next()
                    .map(|node| node.utf8_text(source.as_bytes()).map(ToString::to_string))?;
                let fn_name = capture
                    .nodes_for_capture_index(self.func_name_idx)
                    .next()
                    .map(|node| node.utf8_text(source.as_bytes()).map(ToString::to_string))?;

                match (module, fn_name) {
                    (Ok(module), Ok(function)) => Some(Ok(ExpectedAmLabel { module, function })),
                    (Err(err_mod), _) => {
                        error!("could not fetch the package name: {err_mod}");
                        Some(Err(AmlError::InvalidText))
                    }
                    (_, Err(err_fn)) => {
                        error!("could not fetch the package name: {err_fn}");
                        Some(Err(AmlError::InvalidText))
                    }
                }
            })
            .collect::<std::result::Result<Vec<_>, _>>()
    }
}

#[derive(Debug)]
pub(super) struct AllFunctionsQuery {
    query: Query,
    func_name_idx: u32,
    mod_name_idx: u32,
}

impl AllFunctionsQuery {
    pub fn try_new() -> Result<Self> {
        let query = Query::new(
            language(),
            include_str!("../../runtime/queries/go/all_functions.scm"),
        )?;
        let func_name_idx = query
            .capture_index_for_name(FUNC_NAME_CAPTURE)
            .ok_or_else(|| AmlError::MissingNamedCapture(FUNC_NAME_CAPTURE.to_string()))?;
        let mod_name_idx = query
            .capture_index_for_name(PACK_NAME_CAPTURE)
            .ok_or_else(|| AmlError::MissingNamedCapture(PACK_NAME_CAPTURE.to_string()))?;

        Ok(Self {
            query,
            func_name_idx,
            mod_name_idx,
        })
    }

    pub fn list_function_names(&self, source: &str) -> Result<Vec<ExpectedAmLabel>> {
        let mut parser = new_parser()?;
        let parsed_source = parser.parse(source, None).ok_or(AmlError::Parsing)?;

        let mut cursor = tree_sitter::QueryCursor::new();
        cursor
            .matches(&self.query, parsed_source.root_node(), source.as_bytes())
            .filter_map(|capture| -> Option<Result<ExpectedAmLabel>> {
                let module = capture
                    .nodes_for_capture_index(self.mod_name_idx)
                    .next()
                    .map(|node| node.utf8_text(source.as_bytes()).map(ToString::to_string))?;
                let fn_name = capture
                    .nodes_for_capture_index(self.func_name_idx)
                    .next()
                    .map(|node| node.utf8_text(source.as_bytes()).map(ToString::to_string))?;

                match (module, fn_name) {
                    (Ok(module), Ok(function)) => Some(Ok(ExpectedAmLabel { module, function })),
                    (Err(err_mod), _) => {
                        error!("could not fetch the package name: {err_mod}");
                        Some(Err(AmlError::InvalidText))
                    }
                    (_, Err(err_fn)) => {
                        error!("could not fetch the package name: {err_fn}");
                        Some(Err(AmlError::InvalidText))
                    }
                }
            })
            .collect::<std::result::Result<Vec<_>, _>>()
    }
}
