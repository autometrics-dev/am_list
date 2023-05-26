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
    Rust,
    Go,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let discriminant = s.to_lowercase();
        if ["rust", "rs"].contains(&discriminant.as_str()) {
            return Ok(Self::Rust);
        }

        if discriminant == "go" {
            return Ok(Self::Go);
        }

        Err(format!("Unknown language: {s}"))
    }
}

fn main() -> anyhow::Result<()> {
    let args = Cli::try_parse()?;

    let root = args.root;

    let mut res = match args.language {
        Language::Rust => {
            let mut implementor = am_list::rust::Impl {};
            implementor.list_autometrics_functions(&root)?
        }
        Language::Go => {
            let mut implementor = am_list::go::Impl {};
            implementor.list_autometrics_functions(&root)?
        }
    };

    println!("Autometrics functions in {}:", root.display());
    res.sort();
    for elem in &res {
        println!("{elem}");
    }
    println!("Total: {} functions", res.len());

    Ok(())
}
