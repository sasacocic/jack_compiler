use crate::*;
use std::{
    convert::AsRef,
    env,
    error::Error,
    fs::{read_to_string, File},
    io::{BufRead, BufReader, Bytes, Read, Write},
    iter::Peekable,
    ops::Deref,
    path::{Path, PathBuf},
    process::exit,
};

// for now use a Box<dny Error> but checkout the anyhow crate
type BoxError = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, BoxError>;
type IOResult = std::result::Result<u8, std::io::Error>;

// I want like a string literal type in keyword, because I know it's a fixed number of strings
// I also don't want to do some dumb ass shit like mis-spell something on accident
// was also thinking that a macro could maybe help with compile time check that way I can do
// something like Token::Keyword(class!()) or keyword_token!("classs") - in this case I'd get
// a compiler error because "classs" isn't a keyword
#[derive(Debug)]
enum Token {
    // so keyword
    Keyword(String),
    Symbol(String),
    IntegerConstant(usize),
    StringConstant(String),
    Identifier(String),
}

// basically the tokenizer should return a Vec of the tokens that's it
// this way that the "CompilationEnginer" (aka parser) can easily read those
// tokens
// 1. probably a terrible idea to
struct JackTokenizer {
    f: BufReader<File>,
}

/*
this is pretty annoying I just want something that has
- peek - nth on iterator
- next - next on iterator
 */
impl JackTokenizer {
    fn new(p: PathBuf) -> Result<Self> {
        // hopefully reading from the file is easy - I think it should be, but
        // reading to a string might be easier
        let f = File::open(p)?;
        let f = BufReader::new(f);
        Ok(JackTokenizer { f })
    }

    // want to see the next char.. but for that I have to keep
    // an index into the buffer reader right?
    // fn peek(pos: usize, b: &mut Bytes<BufReader<File>>) -> Result<char> {
    // fn peek(pos: usize, b: &mut impl Iterator<Item = IOResult>) -> Result<char> {
    fn peek(b: &mut Peekable<impl Iterator<Item = IOResult>>) -> Result<char> {
        // let bb = b.nth(self.pos);
        // bb.map(|f| f.map_err(|v| Box::new(v).into()).map(|op| op as char))
        // b.nth(pos).unwrap().map(|o| o as char).map_err(|v| v.into())

        // let vv = b.peek();
        // match vv {
        //     Some(v) => {
        //         let v = v.as_ref();
        //         let j = v.map(|&u8Inside| u8Inside as char);
        //         let b = j.map_err(|err| err.into());
        //         b
        //     }
        //     None => Err("bad".into()),
        // }
        // let v = b.peek().ok_or(Box::new(Err("nothing to peek".into())));
        // b.peek().unwrap().map_err(|e| e.into())
        // let vbb = vv.map_err(|err| err.into());
        // let jj = vbb.map(|ok| ok.clone());

        // let l  = b.peek().unwrap().as_ref().map_err(|op| <std::io::Error as Error>::from(op));
        // map err -> Box<dyn Error>
        let l = b
            .peek()
            // .unwrap()
            .ok_or_else(|| <&str as Into<Box<dyn Error>>>::into("couldn't peek it's an error"))?
            .as_ref()
            .map_err(|mut op: &std::io::Error| {
                let oppp: &dyn Error = op;
                // just returning a string, because somehow &str can turn into -> Box<dyn Error>, but
                // &Error (think &std::io::Error) -> Box<dyn Error> isn't possible, because of lifetime
                "this error instead".into()
                // somehow I need to go from &Error into Box<dyn Error>
            });

        let bb: Result<char> = l.and_then(|inside| Ok(inside.clone().into()));
        // let bb: std::result::Result<char, BoxError> =
        //     l.and_then(|inside| Ok(inside.clone().into()));

        bb
        // let j = l.map_err(|op| op.into());
        // j
    }

    // fn next(pos: &mut usize, b: &mut Bytes<BufReader<File>>) -> Option<Result<char>> {
    fn next(b: &mut impl Iterator<Item = IOResult>) -> Option<Result<char>> {
        // honestly if the cast goes wrong I think this literally panics - so if something
        // isn't utf-8 encoded :(
        // let s = b.next();
        b.next()
            .map(|v| v.map(|vv| vv as char).map_err(|err| err.into()))
    }

    fn tokens_from_string(trimmed_string: &str) -> Vec<Token> {
        // assuming the string will always be trimmed

        trace!("trimmed_string at start: {}", trimmed_string);
        let mut v = Vec::new();

        // split on white space
        let possible_tokens = trimmed_string.split(" ");

        for possible_token in possible_tokens {
            // would this have been easier with regex or macro
            if matches!(
                possible_token,
                "class"
                    | "constructor"
                    | "method"
                    | "function"
                    | "int"
                    | "boolean"
                    | "char"
                    | "void"
                    | "var"
                    | "static"
                    | "let"
                    | "do"
                    | "if"
                    | "else"
                    | "while"
                    | "return"
                    | "true"
                    | "false"
                    | "null"
                    | "this"
                    | "field"
            ) {
                v.push(Token::Keyword(possible_token.to_string()));
            } else if let Ok(int) = trimmed_string.parse::<usize>() {
                v.push(Token::IntegerConstant(int))
            } else if trimmed_string.starts_with('\"') && trimmed_string.ends_with('\"') {
                let string_literal = trimmed_string.get(1..trimmed_string.len());
                match string_literal {
                    Some(string_literal) => {
                        v.push(Token::StringConstant(string_literal.to_string()))
                    }
                    None => panic!("string literal could not be parse"),
                }
            } else {
                // it must be an identifier
                v.push(Token::Identifier(possible_token.to_string()));
            }
        }

        v
    }

    fn tokenize_before_symbol(string: &mut Vec<char>, tokens: &mut Vec<Token>) {
        let trimmed_string = string.iter().collect::<String>();
        let trimmed_string = trimmed_string.trim();
        // dbg!(&tokens);
        if !trimmed_string.is_empty() {
            let mut toks = Self::tokens_from_string(trimmed_string);
            dbg!(&toks);
            tokens.append(toks.as_mut());
            string.clear();
        }
    }

    fn tokenize(self) -> Result<Vec<Token>> {
        let JackTokenizer { f: bytes } = self;
        let mut bytes = bytes.bytes().peekable();
        let mut tokens = Vec::new();

        let mut string = Vec::new();
        loop {
            let next = Self::next(&mut bytes);

            match next {
                None => {
                    // we're done if the next thing is None
                    return Ok(tokens);
                }
                Some(character) => {
                    match character {
                        Err(e) => panic!("shuoldn't happen"),
                        Ok(character) => match character {
                            '{' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("{".into()))
                            }
                            '}' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("}".into()));
                            }
                            '(' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("(".into()));
                            }
                            ')' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol(")".into()));
                            }
                            '[' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("[".into()));
                            }
                            ']' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("]".into()));
                            }
                            '.' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol(".".into()));
                            }
                            ',' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol(",".into()));
                            }
                            ';' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol(";".into()))
                            }
                            '+' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("+".into()));
                            }
                            '-' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("-".into()));
                            }
                            '*' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("*".into()));
                            }
                            '/' => {
                                // this could be division
                                trace!("looking a '/'");
                                if let Ok('/') = Self::peek(&mut bytes) {
                                    // this is a comment so just skip the line
                                    // or just skip the rest of this

                                    trace!("found a comment '//'");
                                    // read until end of line
                                    while let Some(Ok(c)) = Self::next(&mut bytes) {
                                        string.push(c);
                                        if c == '\n' {
                                            // let bb = string.join(' '); // this doesn't work
                                            let bb = string.iter().collect::<String>();
                                            trace!("line: {:?}", &bb);
                                            string.clear();
                                            break;
                                        }
                                    }
                                } else if let Ok('*') = Self::peek(&mut bytes) {
                                    // TODO: test if multiline comments actually work
                                    // read until you see */
                                    trace!("found a comment '/*'");
                                    while let Some(Ok(c)) = Self::next(&mut bytes) {
                                        let char_after_next = Self::peek(&mut bytes);
                                        string.push(c);
                                        // if c == '*' && matches!(char_after_next, Ok('/')) {
                                        if c == '*' && matches!(char_after_next, Ok('/')) {
                                            // let bb = string.join(' '); // this doesn't work
                                            string.push(Self::next(&mut bytes).unwrap().unwrap());
                                            let bb = string.iter().collect::<String>();
                                            trace!("line: {:?}", &bb);
                                            string.clear();
                                            break;
                                        }
                                    }
                                } else {
                                    // it must be division
                                    trace!("found division: add '/' to tokens");
                                    tokens.push(Token::Symbol("/".into()));
                                    string.clear();
                                }
                            }
                            '&' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("&".into()));
                            }
                            '|' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("|".into()));
                            }
                            '<' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("<".into()));
                            }
                            '>' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol(">".into()));
                            }
                            '=' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("=".into()));
                            }
                            '_' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                tokens.push(Token::Symbol("_".into()));
                            }
                            ' ' => {
                                // so what's happening if this is white space?
                                // well it could just be white space at the beginning of a line
                                // or the end of a line or it could be the case that the string
                                // we've been building has a valid token inside of it? What is
                                // the best way to check that?

                                // this should be a trimmed string up to a white space -
                                // it could just be an empty string for example
                                // let trimmed_string = string.iter().collect::<String>();
                                // let trimmed_string = trimmed_string.trim();
                                // // dbg!(&tokens);
                                // if !trimmed_string.is_empty() {
                                //     let mut toks = Self::tokens_from_string(trimmed_string);
                                //     dbg!(&toks);
                                //     tokens.append(toks.as_mut());
                                //     string.clear();
                                // }

                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                // function for figuring out what the possible thing is
                                // then turning it into the correct token
                            }
                            '"' => {
                                Self::tokenize_before_symbol(&mut string, &mut tokens);
                                // kinda not super obvious that the string will be cleared...
                                // but it is - should make that explicit

                                // I don't want the quoted StringConstant
                                // "" is a valid string constant

                                // tokenize_before_symbol handles StringConstant's but it doesn't
                                // really work because parsing it to that point is wrong..

                                trace!("FOUND A STRING");
                                // STOPPED HERE: peek is panicing because I'm unwrapping
                                while let Ok(string_constant_char) = Self::peek(&mut bytes) {
                                    if string_constant_char == '"' {
                                        // found ned of string
                                        Self::next(&mut bytes); // move past final '"'
                                        break;
                                    }
                                    string.push(string_constant_char);
                                    Self::next(&mut bytes);
                                }
                                tokens.push(Token::StringConstant(string.iter().collect()));
                                string.clear();
                            }
                            _ => {
                                // at this point we are dealing with not a symbol
                                // not a comment so we must be dealing with actual characters
                                string.push(character);
                                // since we went through all of the symbols we must be
                                // on something else then
                            }
                        },
                    }
                }
            }
        }
    }

    fn write_to_xml(tokens: Vec<Token>, output_file: Option<&String>) -> Result<()> {
        let mut xml_f = match output_file {
            Some(file_name) => {
                trace!("created (or truncating) to: {:?}", &output_file);
                File::create(PathBuf::from(file_name))
            }
            None => {
                trace!("created (or truncating) to: {:?}", "testing.xml");
                File::create(PathBuf::from("testing.xml"))
            }
        }?;

        xml_f.write("<tokens>\n".as_bytes())?;
        // just an idea is a warp macro basically you give it an
        // identifier then it writes that identifier and evalutaes the
        // value between it for xml_write!(identifier) -> "<identifer> {eval identifier} </identifier>"
        //
        for token in tokens {
            match token {
                Token::Symbol(ident) => match ident.as_str().chars().nth(0) {
                    Some('<') => {
                        xml_f.write("<symbol> &lt; </symbol>\n".as_bytes())?;
                    }
                    Some('>') => {
                        xml_f.write("<symbol> &gt; </symbol>\n".as_bytes())?;
                    }
                    Some('"') => {
                        xml_f.write("<symbol> &quot; </symbol>\n".as_bytes())?;
                    }
                    Some('&') => {
                        xml_f.write("<symbol> &amp; </symbol>\n".as_bytes())?;
                    }
                    _ => {
                        xml_f.write(format!("<symbol> {} </symbol>\n", ident).as_bytes())?;
                    }
                },
                Token::Keyword(keyword) => {
                    xml_f.write(format!("<keyword> {} </keyword>\n", keyword).as_bytes())?;
                }
                Token::IntegerConstant(ident) => {
                    xml_f.write(
                        format!("<integerConstant> {} </integerConstant>\n", ident).as_bytes(),
                    )?;
                }
                Token::StringConstant(ident) => {
                    xml_f.write(
                        format!("<stringConstant> {} </stringConstant>\n", ident).as_bytes(),
                    )?;
                }
                Token::Identifier(ident) => {
                    xml_f.write(format!("<identifier> {} </identifier>\n", ident).as_bytes())?;
                }
            }
        }
        xml_f.write("</tokens>\n".as_bytes())?;
        Ok(())
    }
}

pub fn main() -> Result<()> {
    /*
    TOKENIZER / SCANNER is correct
    - would be good to write some unit tests as this would actually be useful for this
    - also would be good to refactor this code as it's not very easy to understand I think
    - Remove all these comments as they're kinda useless
     */

    // tbh the compile time check isn't even really worth it I think - I don't even
    // know how I could do that - unless I make methods that create
    // let jj = make_token!("keyword", "hello");

    trace!("tokenizer starting");

    let arg: Vec<String> = env::args().collect();

    let jack_file_name = arg.get(1).ok_or("file name not passed")?;
    let output_file_name = arg.get(2);

    // return Err("this is bad".into());
    let mut jack_tokenizer = JackTokenizer::new(PathBuf::from(jack_file_name))?;
    let all_tokens = jack_tokenizer.tokenize()?;
    trace!("all tokens for: {}", jack_file_name);
    dbg!(&all_tokens);
    // can reuse pathbuf with referecne here - I don't think there's any reason
    // it needs to be comsumed
    JackTokenizer::write_to_xml(all_tokens, output_file_name)?;

    Ok(())
}
