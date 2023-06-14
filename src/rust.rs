use crate::{AmlError, ExpectedAmLabel, ListAmFunctions, Result, FUNC_NAME_CAPTURE};
use log::trace;
use rayon::prelude::*;
use std::{
    collections::{HashSet, VecDeque},
    ffi::OsStr,
    fs::read_to_string,
    path::Path,
};
use tree_sitter::{Node, Parser, Query};
use tree_sitter_rust::language;
use walkdir::{DirEntry, WalkDir};

const STRUCT_NAME_CAPTURE: &str = "type.target";
const MOD_NAME_CAPTURE: &str = "mod.name";
const MOD_CONTENTS_CAPTURE: &str = "mod.contents";

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
            .map(|s| s.starts_with('.'))
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
        source_mod_pairs.extend(walker.filter_entry(Self::is_valid).filter_map(|entry| {
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
        }));

        list.par_extend(
            source_mod_pairs
                .par_iter()
                .filter_map(move |(path, module)| {
                    let source = read_to_string(path).ok()?;
                    let am_functions =
                        list_function_names(module.clone(), &source).unwrap_or_default();
                    Some(am_functions)
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

#[derive(Debug)]
struct AmQuery {
    query: Query,
    func_name_idx: u32,
    struct_name_idx: u32,
    mod_name_idx: u32,
    mod_contents_idx: u32,
}

fn query_builder() -> Result<AmQuery> {
    let query = Query::new(
        language(),
        include_str!("../runtime/queries/rust/autometrics.scm"),
    )?;
    let func_name_idx = query
        .capture_index_for_name(FUNC_NAME_CAPTURE)
        .ok_or(AmlError::MissingFuncNameCapture)?;
    let struct_name_idx = query
        .capture_index_for_name(STRUCT_NAME_CAPTURE)
        .ok_or(AmlError::MissingFuncNameCapture)?;
    let mod_name_idx = query
        .capture_index_for_name(MOD_NAME_CAPTURE)
        .ok_or(AmlError::MissingFuncNameCapture)?;
    let mod_contents_idx = query
        .capture_index_for_name(MOD_CONTENTS_CAPTURE)
        .ok_or(AmlError::MissingFuncNameCapture)?;

    Ok(AmQuery {
        query,
        func_name_idx,
        struct_name_idx,
        mod_name_idx,
        mod_contents_idx,
    })
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
    let query = query_builder()?;
    let parsed_source = parser.parse(source, None).ok_or(AmlError::Parsing)?;

    let mut cursor = tree_sitter::QueryCursor::new();
    cursor
        .matches(&query.query, parsed_source.root_node(), source.as_bytes())
        .filter_map(|capture| {
            capture
                .nodes_for_capture_index(query.struct_name_idx)
                .next()
                .map(|node| node.utf8_text(source.as_bytes()).map(ToString::to_string))
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| AmlError::InvalidText)
}

fn list_function_names(module: String, source: &str) -> Result<Vec<ExpectedAmLabel>> {
    fn is_within_mod_item(node: Node, max_parent: Option<Node>, source: &str) -> bool {
        let mut walk = node;
        loop {
            if walk.kind() == "mod_item" {
                trace!(
                    "Node was inside a mod.\nNode:{}\nMax Parent:{}\n",
                    node.utf8_text(source.as_bytes())
                        .map(ToString::to_string)
                        .unwrap(),
                    if let Some(node) = max_parent {
                        node.utf8_text(source.as_bytes())
                            .map(ToString::to_string)
                            .unwrap()
                    } else {
                        source.to_string()
                    }
                );
                break true;
            }
            if let Some(parent) = walk.parent() {
                if max_parent.map_or(false, |max_parent| parent.id() == max_parent.id()) {
                    break false;
                }

                walk = parent;
                continue;
            }
            break false;
        }
    }

    fn list_function_rec(
        current_module: String,
        node: Node,
        query: &AmQuery,
        source: &str,
    ) -> Result<Vec<ExpectedAmLabel>> {
        let mut res = Vec::new();
        let mut cursor = tree_sitter::QueryCursor::new();

        // Detect all functions directly in module scope
        let direct_names = cursor
            .matches(&query.query, node, source.as_bytes())
            .filter_map(|capture| -> Option<Result<(Option<String>, String)>> {
                let fn_node: Node = capture
                    .nodes_for_capture_index(query.func_name_idx)
                    .next()?;

                // Ignore the matches that are within a mod_item, as the recursion will catch it later with the fully qualified module name.
                if is_within_mod_item(fn_node, Some(node), source) {
                    return None;
                }

                let fn_name: std::result::Result<String, std::str::Utf8Error> = fn_node
                    .utf8_text(source.as_bytes())
                    .map(ToString::to_string);
                let struct_name: std::result::Result<Option<String>, std::str::Utf8Error> = capture
                    .nodes_for_capture_index(query.struct_name_idx)
                    .next()
                    .map(|node| node.utf8_text(source.as_bytes()).map(ToString::to_string))
                    .transpose();

                match (struct_name, fn_name) {
                    (Ok(m), Ok(f)) => Some(Ok((m, f))),
                    (Err(_), _) => Some(Err(AmlError::InvalidText)),
                    (_, Err(_)) => Some(Err(AmlError::InvalidText)),
                }
            })
            .collect::<Result<Vec<(_, _)>>>()?;
        res.extend(
            direct_names
                .into_iter()
                .map(|(mod_name, fn_name)| ExpectedAmLabel {
                    module: if let Some(inner) = mod_name {
                        if current_module.is_empty() {
                            inner
                        } else {
                            format!("{current_module}::{inner}")
                        }
                    } else {
                        current_module.clone()
                    },
                    function: fn_name,
                }),
        );

        // Detect all functions in submodule scope
        for capture in cursor.matches(&query.query, node, source.as_bytes()) {
            if let Some(mod_name_node) = capture.nodes_for_capture_index(query.mod_name_idx).next()
            {
                // We only want to consider module nodes that are direct children of the currently iterating node,
                // because the recursion will cleanly look for deeply nested module declarations.
                if mod_name_node
                    .parent()
                    .unwrap_or_else(|| panic!("The rust tree-sitter grammar guarantees that a mod_item:name has a mod_item as parent. {} capture is supposed to capture a mod_item:name", MOD_NAME_CAPTURE))
                    .parent() != Some(node) {
                    continue;
                }

                let mod_name = {
                    if let Ok(val) = mod_name_node
                        .utf8_text(source.as_bytes())
                        .map(ToString::to_string)
                    {
                        val
                    } else {
                        continue;
                    }
                };

                if let Some(contents_node) = capture
                    .nodes_for_capture_index(query.mod_contents_idx)
                    .next()
                {
                    let new_module = if current_module.is_empty() {
                        mod_name
                    } else {
                        format!("{current_module}::{mod_name}")
                    };
                    trace!(
                        "Recursing into mod {}\n{}\n\n\n",
                        new_module,
                        contents_node
                            .utf8_text(source.as_bytes())
                            .map(ToString::to_string)
                            .unwrap()
                    );
                    let inner = list_function_rec(new_module, contents_node, query, source)?;
                    res.extend(inner.into_iter())
                }
            }
        }

        Ok(res)
    }

    let mut parser = new_parser()?;
    let query = query_builder()?;
    let parsed_source = parser.parse(source, None).ok_or(AmlError::Parsing)?;
    list_function_rec(module, parsed_source.root_node(), &query, source)
}

#[cfg(test)]
mod tests;
