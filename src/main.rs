// aka tokenizer
mod scanner;
#[macro_use]
mod utils;

use log::{debug, error, info, trace};

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

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    // has the tokenizer actually tokenize properly I think
    // now I need compare it's inputs and output with the stuff
    // they provide in nand to tetris
    scanner::main()?;
    // let jb: Box<&str> = Box::from("hello");
    // let jb: Box<&str> = "hello".into();
    // let jb: Box<dyn std::error::Error> = "hello".into();
    // let jj = <&str as Into<Box<dyn std::error::Error>>>::into("hello");
    Ok(())
}
