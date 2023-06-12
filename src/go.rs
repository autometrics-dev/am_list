use crate::{AmlError, ExpectedAmLabel, ListAmFunctions, Result, FUNC_NAME_CAPTURE};
use itertools::Itertools;
use rayon::prelude::*;
use std::{collections::HashSet, fs::read_to_string, path::Path};
use tree_sitter::{Parser, Query};
use tree_sitter_go::language;
use walkdir::{DirEntry, WalkDir};

const PACK_NAME_CAPTURE: &str = "pack.name";

#[derive(Clone, Copy, Debug, Default)]
pub struct Impl {}

impl Impl {
    fn is_hidden(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    }

    fn is_valid(entry: &DirEntry) -> bool {
        entry.file_type().is_dir()
            || !Impl::is_hidden(entry)
                && entry
                    .file_name()
                    .to_str()
                    .map(|s| s.ends_with(".go"))
                    .unwrap_or(false)
    }
}

impl ListAmFunctions for Impl {
    fn list_autometrics_functions(&mut self, project_root: &Path) -> Result<Vec<ExpectedAmLabel>> {
        const PREALLOCATED_ELEMS: usize = 100;
        let mut list = HashSet::with_capacity(PREALLOCATED_ELEMS);

        let walker = WalkDir::new(project_root).into_iter();
        let mut source_mod_pairs = Vec::with_capacity(PREALLOCATED_ELEMS);
        source_mod_pairs.extend(walker.filter_entry(Self::is_valid).filter_map(|entry| {
            let entry = entry.ok()?;
            Some(
                entry
                    .path()
                    .to_str()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            )
        }));

        list.par_extend(source_mod_pairs.par_iter().filter_map(move |path| {
            let source = read_to_string(path).ok()?;
            let names = list_function_names(&source).unwrap_or_default();
            Some(
                names
                    .into_iter()
                    .map(move |(module, function)| ExpectedAmLabel { module, function })
                    .collect::<Vec<_>>(),
            )
        }));

        let mut result = Vec::with_capacity(PREALLOCATED_ELEMS);
        result.extend(list.into_iter().flatten());
        Ok(result)
    }

    fn list_all_functions(&mut self, _project_root: &Path) -> Result<Vec<ExpectedAmLabel>> {
        unimplemented!("listing all functions in Golang")
    }
}

fn new_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(language())?;
    Ok(parser)
}

fn query_builder() -> Result<(Query, u32, u32)> {
    let query = Query::new(
        language(),
        include_str!("../runtime/queries/go/autometrics.scm"),
    )?;
    let idx = query
        .capture_index_for_name(FUNC_NAME_CAPTURE)
        .ok_or_else(|| AmlError::MissingNamedCapture(FUNC_NAME_CAPTURE.into()))?;
    let mod_idx = query
        .capture_index_for_name(PACK_NAME_CAPTURE)
        .ok_or_else(|| AmlError::MissingNamedCapture(PACK_NAME_CAPTURE.into()))?;
    Ok((query, idx, mod_idx))
}

fn list_function_names(source: &str) -> Result<Vec<(String, String)>> {
    let mut parser = new_parser()?;
    let (query, idx, mod_idx) = query_builder()?;
    let parsed_source = parser.parse(source, None).ok_or(AmlError::Parsing)?;

    let mut cursor = tree_sitter::QueryCursor::new();
    // TODO(maint): the complexity/type tetris needs to go down.
    cursor
        .matches(&query, parsed_source.root_node(), source.as_bytes())
        .map(|capture| {
            let module = capture
                .nodes_for_capture_index(mod_idx)
                .next()
                .map(|node| node.utf8_text(source.as_bytes()).map(ToString::to_string))
                .transpose()?;
            let fn_name = capture
                .nodes_for_capture_index(idx)
                .next()
                .map(|node| node.utf8_text(source.as_bytes()).map(ToString::to_string))
                .transpose()?;
            Ok((module, fn_name))
        })
        .filter_map_ok(|(module, fn_name)| Some((module?, fn_name?)))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_: anyhow::Error| AmlError::InvalidText)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_simple() {
        let source = r#"
        package lambda

        //autometrics:inst
        func the_one() {
        	return nil
        }
        "#;

        let list = list_function_names(source).unwrap();

        assert_eq!(list.len(), 1);
        assert_eq!(list[0], ("lambda".to_string(), "the_one".to_string()));
    }

    #[test]
    fn detect_legacy() {
        let source = r#"
        package lambda

        func not_the_one() {
        }

        //autometrics:doc
        func sandwiched_function() {
        	return nil
        }

        func not_that_one_either() {
        }
        "#;

        let list = list_function_names(source).unwrap();

        assert_eq!(list.len(), 1);
        assert_eq!(
            list[0],
            ("lambda".to_string(), "sandwiched_function".to_string())
        );
    }
}
