use crate::{AmlError, ExpectedAmLabel, ListAmFunctions, Result, FUNC_NAME_CAPTURE};
use std::{ffi::OsStr, fs::read_to_string, path::Path};
use tree_sitter::{Parser, Query};
use tree_sitter_rust::language;
use walkdir::{DirEntry, WalkDir};

#[derive(Clone, Copy, Debug, Default)]
pub struct Impl {}

impl Impl {
    fn is_hidden(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
    }

    fn is_valid(entry: &DirEntry) -> bool {
        entry.file_type().is_dir()
            || !Impl::is_hidden(entry)
                && entry
                    .file_name()
                    .to_str()
                    .map(|s| s.ends_with(".rs"))
                    .unwrap_or(false)
    }

    fn module_name(entry: &DirEntry) -> String {
        let file_candidate = entry
            .file_name()
            .to_str()
            .and_then(|s| s.strip_suffix(".rs"))
            .map(ToString::to_string)
            .unwrap_or_default();

        if !file_candidate.is_empty() && &file_candidate != "mod" {
            return file_candidate;
        }

        entry
            .path()
            .parent()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .map(ToString::to_string)
            .unwrap_or_default()
    }
}

impl ListAmFunctions for Impl {
    fn list_autometrics_functions(&self, project_root: &Path) -> Result<Vec<ExpectedAmLabel>> {
        let mut list = Vec::new();
        let walker = WalkDir::new(project_root).into_iter();
        // TODO(perf): parallelize this extend
        list.extend(
            walker
                .filter_entry(|e| Self::is_valid(e))
                .into_iter()
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let module = Self::module_name(&entry);
                    read_to_string(entry.path()).ok().map(|s| (s, module))
                })
                .map(move |(source, module)| {
                    let names = list_function_names(&source).unwrap_or_default();
                    names.into_iter().map(move |fn_name| ExpectedAmLabel {
                        module: module.clone(),
                        function: fn_name,
                    })
                }).flatten(),
        );
        Ok(list)
    }
}

fn new_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(language())?;
    Ok(parser)
}

fn query_builder() -> Result<(Query, u32)> {
    let query = Query::new(
        language(),
        include_str!("../runtime/queries/rust/autometrics.scm"),
    )?;
    let idx = query
        .capture_index_for_name(FUNC_NAME_CAPTURE)
        .ok_or(AmlError::MissingFuncNameCapture)?;
    Ok((query, idx))
}

pub fn list_function_names(source: &str) -> Result<Vec<String>> {
    let mut parser = new_parser()?;
    let (query, idx) = query_builder()?;
    let parsed_source = parser.parse(source, None).ok_or(AmlError::Parsing)?;

    let mut cursor = tree_sitter::QueryCursor::new();
    cursor
        .matches(&query, parsed_source.root_node(), source.as_bytes())
        .filter_map(|capture| {
            capture
                .nodes_for_capture_index(idx)
                .next()
                .map(|node| node.utf8_text(source.as_bytes()).map(ToString::to_string))
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| AmlError::InvalidText)
}
