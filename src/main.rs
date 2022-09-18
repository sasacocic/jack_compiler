type BoxError = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, BoxError>;
type IOResult = std::result::Result<u8, std::io::Error>;
// aka tokenizer
mod parser;
mod scanner;
#[macro_use]
mod utils;

use clap::Parser;
use log::{debug, info, trace};
use std::fmt::Debug;
use std::str::FromStr;

/*
- add logging - good for debugging
- use Rusts built in unit testing functionality. Will be
really good to test this kind of stuff easily.
 */

/*
what I need to do
1. produce a file that looks like

looks like xml
...

<keyword> if </keyword>
..
<symbol> ) </symbol>
..
<identifier> sign </identifier>

 */

enum CompilerPart {
    Parser,
    Scanner,
}

// better error handling???
// needed for clap derive
impl FromStr for CompilerPart {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "scanner" => Ok(CompilerPart::Scanner),
            "parser" => Ok(CompilerPart::Parser),
            _ => Err("bad thing".to_string()),
        }
    }
}

// very simple right now - should provide some guidance on how to use this
// 1. I want to be able to input strings from the terminal and get them back tokenized
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    part: CompilerPart,
    #[clap(value_parser)]
    jack_file: Option<String>,
    // should only be present if part == "scanner"
    tokenizer_output_file_name: Option<String>,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    // has the tokenizer actually tokenize properly I think
    // now I need compare it's inputs and output with the stuff
    // they provide in nand to tetris
    // let jb: Box<&str> = Box::from("hello");
    // let jb: Box<&str> = "hello".into();
    // let jb: Box<dyn std::error::Error> = "hello".into();
    // let jj = <&str as Into<Box<dyn std::error::Error>>>::into("hello");

    // take cli arguments

    let args = Args::parse();

    match args.part {
        CompilerPart::Parser => match args.jack_file {
            Some(jack_file_path) => {
                parser::main(jack_file_path)?;
            }
            None => {
                parser::repl()?;
            }
        },
        CompilerPart::Scanner => match args.jack_file {
            Some(jack_file_path) => {
                scanner::main(jack_file_path, args.tokenizer_output_file_name)?;
            }
            None => {
                // apparently I can't tokenize: "1 + 2 + 3" properly... figure that out
                scanner::repl()?;
            }
        },
    }

    Ok(())
}
