#[macro_export]
macro_rules! make_keyword {
    ( "class" ) => {
        Token::Keyword("class".into())
    };
    ( "constructor" ) => {
        Token::Keyword("constructor".into())
    };
    ( "function" ) => {
        Token::Keyword("function".into())
    };
    ( "method" ) => {
        Token::Keyword("method".into())
    };
    ( "field" ) => {
        Token::Keyword("field".into())
    };
    ( "static" ) => {
        Token::Keyword("static".into())
    };
    ( "var" ) => {
        Token::Keyword("var".into())
    };
    ( "int" ) => {
        Token::Keyword("int".into())
    };
    ( "char" ) => {
        Token::Keyword("char".into())
    };
    ( "boolean" ) => {
        Token::Keyword("boolean".into())
    };
    ( "void" ) => {
        Token::Keyword("void".into())
    };
    ( "true" ) => {
        Token::Keyword("true".into())
    };
    ( "false" ) => {
        Token::Keyword("false".into())
    };
    ( "null" ) => {
        Token::Keyword("null".into())
    };
    ( "this" ) => {
        Token::Keyword("this".into())
    };
    ( "let" ) => {
        Token::Keyword("let".into())
    };
    ( "do" ) => {
        Token::Keyword("do".into())
    };
    ( "if" ) => {
        Token::Keyword("if".into())
    };
    ( "else" ) => {
        Token::Keyword("else".into())
    };
    ( "while" ) => {
        Token::Keyword("while".into())
    };
    ( "return" ) => {
        Token::Keyword("return".into())
    };
}

#[macro_export]
macro_rules! make_symbol {
    ("{") => {
        Token::Symbol("{".into())
    };
    ("}") => {
        Token::Symbol("}".into())
    };
    ("(") => {
        Token::Symbol("(".into())
    };
    (")") => {
        Token::Symbol(")".into())
    };
    ("[") => {
        Token::Symbol("[".into())
    };
    ("]") => {
        Token::Symbol("]".into())
    };
    (".") => {
        Token::Symbol(".".into())
    };
    (",") => {
        Token::Symbol(",".into())
    };
    (";") => {
        Token::Symbol(";".into())
    };
    ("+") => {
        Token::Symbol("+".into())
    };
    ("-") => {
        Token::Symbol("-".into())
    };
    ("*") => {
        Token::Symbol("*".into())
    };
    ("/") => {
        Token::Symbol("/".into())
    };
    ("&") => {
        Token::Symbol("&".into())
    };
    ("|") => {
        Token::Symbol("|".into())
    };
    ("<") => {
        Token::Symbol("<".into())
    };
    (">") => {
        Token::Symbol(">".into())
    };
    ("=") => {
        Token::Symbol("=".into())
    };
    ("_") => {
        Token::Symbol("_".into())
    };
}

#[macro_export]
macro_rules! make_token {
    ("keyword", $e:tt) => {{
        crate::make_keyword!($e) // for whatever reason TokenTree can be passed how I expect...
    }};
    ("symbol", $e:tt) => {{
        make_symbol!($e)
        // TODO: encode all symbols here
    }};
    ("intConstant", $e:expr) => {{
        Token::IntegerConstant($e)
    }};
    ("stringConstant", $e:expr) => {{
        Token::StringConstant($e)
    }};
    ("identifier", $e:expr ) => {{
        Token::Identifier($e)
    }};
}
