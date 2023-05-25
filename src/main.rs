use anyhow::anyhow;
use clap::Parser;
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "SOURCE_FILE")]
    source_file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::try_parse()?;

    let source = fs::read_to_string(&args.source_file)?;

    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_rust::language();
    parser.set_language(language)?;

    let parsed_source = parser.parse(&source, None).ok_or(anyhow!("Syntax error"))?;

    let all_functions_query =
        tree_sitter::Query::new(language, "(function_item name: (identifier) @func.name)")?;
    let am_functions_query = tree_sitter::Query::new(language, "((attribute_item (attribute (identifier) @attr)) . (function_item name: (identifier) @func.name) (#eq? @attr \"autometrics\"))")?;
    let am_fn_idx = am_functions_query
        .capture_index_for_name("func.name")
        .ok_or(anyhow!("Bad capture name"))?;
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut am_cursor = tree_sitter::QueryCursor::new();

    let all_functions_matches = cursor.captures(
        &all_functions_query,
        parsed_source.root_node(),
        source.as_bytes(),
    );
    let am_functions_matches = am_cursor.matches(
        &am_functions_query,
        parsed_source.root_node(),
        source.as_bytes(),
    );

    println!("All functions in {}:", args.source_file.display());
    for capture in all_functions_matches {
        for hit in capture.0.captures {
            println!("{}", hit.node.utf8_text(source.as_bytes())?);
        }
    }

    println!("Autometrics functions in {}:", args.source_file.display());
    for capture in am_functions_matches {
        println!(
            "{}",
            capture
                .nodes_for_capture_index(am_fn_idx)
                .next()
                .unwrap()
                .utf8_text(source.as_bytes())?
        );
    }

    Ok(())
}
