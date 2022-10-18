type IOResult = std::result::Result<u8, std::io::Error>;
// aka tokenizer
mod parser;
mod scanner;
#[macro_use]
mod utils;
mod cmd;
use cmd::{DebugSubCmd, LogLevels, Mode, Part, Repl};

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::{debug, trace};
use std::fmt::Debug;
use std::str::FromStr;

/*
TODOS:
1. add logging
- I need to know what the fuck this application is doing as it's doing it - I guess I can debug, but as it stands I'd rather rely on logging
2. clean up tests
- right now they are a mess. They should be here to help me make sure my code is actually correct.
3. Better error handling
- ATM just have more meaningful error handling. Eventually I'll get to how to handle errors in the parser / tokenizer, but for now
just make it better than what it is now.
4. Think about commiting the testing floder, becase I'm acutally using it to test stuff

 */

#[derive(Debug)]
enum CatchAllError {
    Generic(String),
}

impl std::fmt::Display for CatchAllError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CatchAllError::Generic(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for CatchAllError {}

#[derive(Parser)]
#[command(name = "jack")]
#[command(author = "Sasa Cocic-Banjanac")]
#[command(version = "0.0.1")]
#[command(about = "compile and run programs written in the Jack programming language")]
struct JackCompiler {
    #[arg(short = 'l', long = "log-level")]
    log_level: Option<LogLevels>,
    #[command(subcommand)]
    mode: Mode,
}

impl JackCompiler {
    fn run(self) -> Result<()> {
        // TODO:  run the modes from here?
        match self.mode {
            Mode::Debug(DebugSubCmd {
                format: _,
                output_file,
                file,
                part,
            }) => match part {
                Part::Scanner => {
                    scanner::main(file, output_file)?;
                }
                Part::Parser => {
                    parser::main(file)?;
                }
            },
            Mode::Run => {
                unimplemented!("haven't completed run command yet, but I shoudl?");
                /*
                TODO
                - basically this needs to look at all the jack files in the directory that it's in, compile them, and execute them
                 */
            }
            Mode::Repl(Repl { part }) => match part {
                Part::Scanner => scanner::repl()?,
                Part::Parser => parser::repl()?,
            },
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = JackCompiler::parse();

    match &args.log_level {
        Some(level) => {
            let level = match level {
                LogLevels::Debug => "DEBUG",
                LogLevels::Info => "INFO",
                LogLevels::Trace => "TRACE",
            };
            env_logger::builder()
                .default_format()
                .filter_level(log::LevelFilter::from_str(level).expect("a log level"))
                .init();
        }
        None => env_logger::builder()
            .default_format()
            .filter_level(log::LevelFilter::Debug)
            .init(),
    }

    args.run()
}
