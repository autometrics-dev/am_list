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

const ANNOTATED_IMPL_NAME_CAPTURE: &str = "type.impl";
const ANNOTATED_IMPL_METHOD_NAME_CAPTURE: &str = "inner.func.name";
const MOD_NAME_CAPTURE: &str = "mod.name";
const MOD_CONTENTS_CAPTURE: &str = "mod.contents";
const IMPL_NAME_CAPTURE: &str = "impl.type";
const IMPL_CONTENTS_CAPTURE: &str = "impl.contents";

const GRAMMAR_IMPL_ITEM_NODE_KIND: &str = "impl_item";
const GRAMMAR_MOD_ITEM_NODE_KIND: &str = "mod_item";

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
                        list_function_names(module.clone(), &source, true).unwrap_or_default();
                    Some(am_functions)
                }),
        );

        let mut result = Vec::with_capacity(PREALLOCATED_ELEMS);
        result.extend(list.into_iter().flatten());
        Ok(result)
    }

    fn list_all_functions(&mut self, project_root: &Path) -> Result<Vec<ExpectedAmLabel>> {
        const PREALLOCATED_ELEMS: usize = 400;
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
                        list_function_names(module.clone(), &source, false).unwrap_or_default();
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
    /// Index of the capture for a function name.
    func_name_idx: u32,
    /// Index of the capture for the type name of an `#[autometrics]`-annotated impl block
    ///
    /// This is an option, because when we want to list all functions, we do not want to use
    /// this capture ever (we will instead recurse into every impl block.)
    annotated_impl_type_name_idx: Option<u32>,
    /// Index of the capture for a method name within an `#[autometrics]`-annotated impl block
    ///
    /// This is an option, because when we want to list all functions, we do not want to use
    /// this capture ever (we will instead recurse into every impl block.)
    annotated_impl_method_name_idx: Option<u32>,
    /// Index of the capture for the name of a module that is defined in file.
    mod_name_idx: u32,
    /// Index of the capture for the contents of a module that is defined in file.
    mod_contents_idx: u32,
    /// Index of a capture for the type name associated to any impl block in the file.
    impl_type_idx: u32,
    /// Index of a capture for the contents (the declarations) associated to any impl block in the file.
    impl_contents_idx: u32,
}

fn query_builder(has_autometrics: bool) -> Result<AmQuery> {
    let query = if has_autometrics {
        Query::new(
            language(),
            include_str!("../runtime/queries/rust/autometrics.scm"),
        )?
    } else {
        Query::new(
            language(),
            include_str!("../runtime/queries/rust/all_functions.scm"),
        )?
    };
    let func_name_idx = query
        .capture_index_for_name(FUNC_NAME_CAPTURE)
        .ok_or_else(|| AmlError::MissingFuncNameCapture(FUNC_NAME_CAPTURE.into()))?;
    let annotated_impl_type_name_idx = query.capture_index_for_name(ANNOTATED_IMPL_NAME_CAPTURE);
    let annotated_impl_method_name_idx =
        query.capture_index_for_name(ANNOTATED_IMPL_METHOD_NAME_CAPTURE);
    let mod_name_idx = query
        .capture_index_for_name(MOD_NAME_CAPTURE)
        .ok_or_else(|| AmlError::MissingFuncNameCapture(MOD_NAME_CAPTURE.into()))?;
    let mod_contents_idx = query
        .capture_index_for_name(MOD_CONTENTS_CAPTURE)
        .ok_or_else(|| AmlError::MissingFuncNameCapture(MOD_NAME_CAPTURE.into()))?;
    let impl_type_idx = query
        .capture_index_for_name(IMPL_NAME_CAPTURE)
        .ok_or_else(|| AmlError::MissingFuncNameCapture(IMPL_NAME_CAPTURE.into()))?;
    let impl_contents_idx = query
        .capture_index_for_name(IMPL_CONTENTS_CAPTURE)
        .ok_or_else(|| AmlError::MissingFuncNameCapture(IMPL_CONTENTS_CAPTURE.into()))?;

    Ok(AmQuery {
        query,
        func_name_idx,
        annotated_impl_type_name_idx,
        annotated_impl_method_name_idx,
        mod_name_idx,
        mod_contents_idx,
        impl_type_idx,
        impl_contents_idx,
    })
}

fn list_function_names(
    module: String,
    source: &str,
    has_autometrics: bool,
) -> Result<Vec<ExpectedAmLabel>> {
    fn is_within_mod_item(node: Node, max_parent: Option<Node>, source: &str) -> bool {
        let mut walk = node;
        loop {
            if walk.kind() == GRAMMAR_MOD_ITEM_NODE_KIND {
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

    fn is_within_impl_item(node: Node, max_parent: Option<Node>, source: &str) -> bool {
        let mut walk = node;
        loop {
            if walk.kind() == GRAMMAR_IMPL_ITEM_NODE_KIND {
                trace!(
                    "Node was inside a impl block.\nNode:{}\nMax Parent:{}\n",
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

        // TODO(maint): the direct_names and impl_block_methods block could be factorized in one function with different arguments (an AmQuery method)

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

                // Ignore the matches that are within a impl_item, as the impl_block_names variable below catches those, applying the
                // fully qualified module name.
                if is_within_impl_item(fn_node, Some(node), source) {
                    return None;
                }

                let fn_name: std::result::Result<String, std::str::Utf8Error> = fn_node
                    .utf8_text(source.as_bytes())
                    .map(ToString::to_string);
                let struct_name: std::result::Result<Option<String>, std::str::Utf8Error> = query
                    .annotated_impl_type_name_idx
                    .and_then(|idx| capture.nodes_for_capture_index(idx).next())
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

        if let Some(impl_block_idx) = query.annotated_impl_type_name_idx {
            if query.annotated_impl_method_name_idx.is_none() {
                return Err(AmlError::MissingFuncNameCapture(
                    ANNOTATED_IMPL_METHOD_NAME_CAPTURE.into(),
                ));
            }
            // Detect all methods from annotated impl blocks directly in module scope
            let impl_block_methods = cursor
                .matches(&query.query, node, source.as_bytes())
                .filter_map(|capture| -> Option<Result<(Option<String>, String)>> {
                    let fn_node: Node = capture
                        .nodes_for_capture_index(
                            query
                                .annotated_impl_method_name_idx
                                .expect("The None case has been handled just before"),
                        )
                        .next()?;

                    // Ignore the matches that are within a mod_item, as the recursion will catch it later with the fully qualified module name.
                    if is_within_mod_item(fn_node, Some(node), source) {
                        return None;
                    }

                    let fn_name: std::result::Result<String, std::str::Utf8Error> = fn_node
                        .utf8_text(source.as_bytes())
                        .map(ToString::to_string);
                    let struct_name: std::result::Result<Option<String>, std::str::Utf8Error> =
                        capture
                            .nodes_for_capture_index(impl_block_idx)
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
                impl_block_methods
                    .into_iter()
                    .map(|(struct_name, fn_name)| ExpectedAmLabel {
                        module: if let Some(inner) = struct_name {
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
        } else {
            trace!("Skipping direct impl block detection, because current query does not have a {ANNOTATED_IMPL_NAME_CAPTURE}.")
        }

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

            if let Some(impl_type_node) =
                capture.nodes_for_capture_index(query.impl_type_idx).next()
            {
                // We only want to consider impl blocks that are direct children of the currently iterating node,
                // because the recursion will cleanly look for deeply nested module declarations.
                if impl_type_node
                    .parent()
                    .unwrap_or_else(|| panic!("The rust tree-sitter grammar guarantees that a impl_item:type_identifier has a impl_item as parent. {} capture is supposed to capture a mod_item:name", MOD_NAME_CAPTURE))
                    .parent() != Some(node) {
                    continue;
                }

                let type_name = {
                    if let Ok(val) = impl_type_node
                        .utf8_text(source.as_bytes())
                        .map(ToString::to_string)
                    {
                        val
                    } else {
                        continue;
                    }
                };

                if let Some(contents_node) = capture
                    .nodes_for_capture_index(query.impl_contents_idx)
                    .next()
                {
                    let new_module = if current_module.is_empty() {
                        type_name
                    } else {
                        format!("{current_module}::{type_name}")
                    };
                    trace!(
                        "Recursing into impl block {}\n{}\n\n\n",
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
    let query = query_builder(has_autometrics)?;
    let parsed_source = parser.parse(source, None).ok_or(AmlError::Parsing)?;
    list_function_rec(module, parsed_source.root_node(), &query, source)
}

#[cfg(test)]
mod tests;
