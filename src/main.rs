use am_list::ListAmFunctions;
use clap::Parser;
use std::{path::PathBuf, str::FromStr};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "LANGUAGE")]
    language: Language,
    #[arg(value_name = "ROOT")]
    root: PathBuf,
}

#[derive(Clone, Copy)]
enum Language {
    Rust
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if ["rust", "rs"].contains(&s.to_lowercase().as_str()) {
            return Ok(Self::Rust)
        }

        Err(format!("Unknown language: {s}"))
    }
}

fn main() -> anyhow::Result<()> {
    let args = Cli::try_parse()?;

    let root = args.root;

    let implementor = am_list::rust::Impl{};

    let res = implementor.list_autometrics_functions(&root)?;

    println!("Autometrics functions in {}:", root.display());
    for elem in &res {
        println!("{elem}");
    }
    println!("Total: {} functions", res.len());

    Ok(())
}
