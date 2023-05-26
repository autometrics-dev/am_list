use crate::{AmlError, ExpectedAmLabel, ListAmFunctions, Result, FUNC_NAME_CAPTURE};
use rayon::prelude::*;
use std::{
    collections::{HashSet, VecDeque},
    ffi::OsStr,
    fs::read_to_string,
    path::Path,
};
use tree_sitter::{Parser, Query};
use tree_sitter_rust::language;
use walkdir::{DirEntry, WalkDir};

const STRUCT_NAME_CAPTURE: &str = "type.target";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AmStruct {
    module: String,
    strc: String,
}

// TODO: Add state in the impl to allow remembering structs that have
// the decoration in different files from the impl blocks
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

    fn fully_qualified_module_name(entry: &DirEntry) -> String {
        let mut current_depth = entry.depth();
        let mut mod_name_elements = VecDeque::with_capacity(8);
        let mut path = entry.path();

        // NOTE(magic)
        // This "1" magic constant bears the assumption "am_list" is called
        // from the root of a crate.
        while current_depth > 1 {
            if path.is_dir() {
                if let Some(component) = path.file_name() {
                    mod_name_elements.push_front(component.to_string_lossy().to_string());
                }
            } else if path.is_file() {
                if let Some(stem) = path
                    .file_name()
                    .and_then(|os_str| os_str.to_str())
                    .and_then(|file_name| file_name.strip_suffix(".rs"))
                {
                    if stem != "mod" {
                        mod_name_elements.push_front(stem.to_string());
                    }
                }
            }

            if path.parent().is_some() {
                path = path.parent().unwrap();
                current_depth -= 1;
            } else {
                break;
            }
        }

        itertools::intersperse(mod_name_elements.into_iter(), "::".to_string()).collect()
    }
}

impl ListAmFunctions for Impl {
    fn list_autometrics_functions(&mut self, project_root: &Path) -> Result<Vec<ExpectedAmLabel>> {
        const PREALLOCATED_ELEMS: usize = 100;
        let mut list = HashSet::with_capacity(PREALLOCATED_ELEMS);

        let walker = WalkDir::new(project_root).into_iter();
        let mut source_mod_pairs = Vec::with_capacity(PREALLOCATED_ELEMS);
        source_mod_pairs.extend(
            walker
                .filter_entry(|e| Self::is_valid(e))
                .into_iter()
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let module = Self::fully_qualified_module_name(&entry);
                    Some((
                        entry
                            .path()
                            .to_str()
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                        module,
                    ))
                }),
        );

        list.par_extend(
            source_mod_pairs
                .par_iter()
                .filter_map(move |(path, module)| {
                    let source = read_to_string(path).ok()?;
                    let names = list_function_names(&source).unwrap_or_default();
                    Some(
                        names
                            .into_iter()
                            .map(move |fn_name| ExpectedAmLabel {
                                module: module.clone(),
                                function: fn_name,
                            })
                            .collect::<Vec<_>>(),
                    )
                }),
        );

        let mut result = Vec::with_capacity(PREALLOCATED_ELEMS);
        result.extend(list.into_iter().flatten());
        Ok(result)
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
        include_str!("../runtime/queries/rust/autometrics.scm"),
    )?;
    let idx = query
        .capture_index_for_name(FUNC_NAME_CAPTURE)
        .ok_or(AmlError::MissingFuncNameCapture)?;
    let cap_idx = query
        .capture_index_for_name(STRUCT_NAME_CAPTURE)
        .ok_or(AmlError::MissingFuncNameCapture)?;
    Ok((query, idx, cap_idx))
}

// CLIPPY: we allow this unused function here to serve as a starting
// point to deal with the "struct and impl in different files" issue
#[allow(dead_code)]
fn known_struct_query_builder(annotated_struct_list: &HashSet<AmStruct>) -> Result<(Query, u32)> {
    let regex = itertools::intersperse(annotated_struct_list.iter().map(|p| p.strc.as_str()), "|")
        .collect::<String>();
    let query_src = format!(
        include_str!("../runtime/queries/rust/am_struct.scm.tpl"),
        regex
    );
    println!("{query_src}");

    let query = Query::new(language(), &query_src)?;
    let idx = query
        .capture_index_for_name(FUNC_NAME_CAPTURE)
        .ok_or(AmlError::MissingFuncNameCapture)?;
    Ok((query, idx))
}

// CLIPPY: we allow this unused function here to serve as a starting
// point to deal with the "struct and impl in different files" issue
#[allow(dead_code)]
fn list_struct_names(source: &str) -> Result<Vec<String>> {
    let mut parser = new_parser()?;
    let (query, _, idx) = query_builder()?;
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

fn list_function_names(source: &str) -> Result<Vec<String>> {
    let mut parser = new_parser()?;
    let (query, idx, _) = query_builder()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_single() {
        let source = r#"
        #[autometrics]
        fn main() {}
        "#;

        let list = list_function_names(source).unwrap();

        assert_eq!(list.len(), 1);
        assert_eq!(list[0], "main");
    }

    #[test]
    fn detect_impl_block() {
        let source = r#"
        #[autometrics]
        struct Foo{};

        impl Foo {
            fn method_a() {}
        }
        "#;

        let list = list_function_names(source).unwrap();

        assert_eq!(list.len(), 1);
        assert_eq!(list[0], "method_a");
    }

    #[test]
    fn detect_struct_annotation() {
        let source = r#"
        #[autometrics]
        struct Foo{};
        "#;

        let list = list_struct_names(source).unwrap();

        assert_eq!(list.len(), 1);
        assert_eq!(list[0], "Foo");
    }

    #[test]
    fn detect_trait_impl_block() {
        let source = r#"
        #[autometrics]
        struct Foo{};

        impl A for Foo {
            fn m_a() {}
        }
        "#;

        let list = list_function_names(source).unwrap();

        assert_eq!(list.len(), 1);
        assert_eq!(list[0], "m_a");
    }

    #[test]
    fn dodge_wrong_impl_block() {
        let source = r#"
        #[autometrics]
        struct Foo{};

        struct Bar{};

        impl Bar {
            fn method_one() {}
        }
        impl Foo {
            fn method_two() {}
        }
        impl Bar {
            fn method_three() {}
        }
        impl Foo {
            fn method_four() {}
        }
        "#;

        let list = list_function_names(source).unwrap();

        assert_eq!(list.len(), 2);
        assert!(list.contains(&"method_two".to_string()));
        assert!(list.contains(&"method_four".to_string()));
    }
}
