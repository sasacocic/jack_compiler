use crate::*;
use std::path::PathBuf;

#[derive(Subcommand, Clone)]
#[command(author = "Sasa Cocic-Banjanac")]
#[command(version = "0.0.1")]
#[command(about = "mode to run jack compiler")]
pub enum Mode {
    #[command(about = "compile and execute the jack files in this directory")]
    Run,
    #[command(about = "debug the jack scanner or parser")]
    Debug(DebugSubCmd),
    #[command(about = "read print evaulate - can be used with scanner & parser")]
    Repl(Repl),
}

#[derive(Args, Clone)]
pub struct Repl {
    #[arg(short = 'd', long = "debug")]
    pub part: Part,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Part {
    Scanner,
    Parser,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Format {
    Xml,
    RustDisplay,
}

#[derive(Args, Clone)]
pub struct DebugSubCmd {
    pub part: Part,
    pub file: PathBuf,
    #[arg(short = 'f', long = "format")]
    pub format: Option<Format>,
    #[arg(short = 'o', long = "output-file", default_value = "thing.out.jack")]
    pub output_file: Option<PathBuf>,
}

#[derive(ValueEnum, Clone)]
pub enum LogLevels {
    Trace,
    Info,
    Debug,
}
