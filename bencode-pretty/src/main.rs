use std::fs::read;
use std::io::Read as _;
use std::path::PathBuf;

use anyhow::{Context, Result};
use bendy::inspect::*;
use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(version, about="bencode-pretty\n\n\
    Pretty-prints bencode provided either through stdin or as a list of file paths.\n\
    If using std-in, only one unit (dict, list, int or byte string) will be pretty printed.")]
struct Args {
    /// Print the pretty printed bencode as a rust string literal.
    /// Default: false
    #[arg(short, long)]
    string_literal: bool,

    /// List of file paths to read bencode from for pretty printing.
    /// Listens to stdin if no file paths are provided.
    file_paths: Vec<PathBuf>
}

fn main() -> Result<()> {
    let args = Args::parse();
    let as_string_literal = args.string_literal;
    if args.file_paths.is_empty() {
        let mut input = Vec::new();
        std::io::stdin().lock().read_to_end(&mut input)?;
        pretty_print(
            input.as_slice(),
            as_string_literal,
            "stdin",
        )?;
    } else {
        for p in args.file_paths {
            let contents = read(&p)
                .context(format!("Could not read file from path: {:?}", &p))?;
            pretty_print(
                contents.as_slice(),
                as_string_literal,
                p.display().to_string().as_str(),
            )?;
        }
    }

    Ok(())
}

fn pretty_print(i: &[u8], as_string_literal: bool, source: &str) -> Result<()> {
    let i = Inspectable::try_from(i)
        .context(format!("Could not parse {:?} as bencode", source))?;
    println!("{}", if as_string_literal {
        i.as_rust_string_literal()
    } else {
        i.as_pretty_printed()
    });
    Ok(())
}
