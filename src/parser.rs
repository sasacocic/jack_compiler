use core::panic;
use std::any::Any;
use std::fs::File;
use std::io::{stdin, Write};
use std::iter::Peekable;
use std::path::PathBuf;
use std::{vec};

use crate::*;

use crate::scanner::{JackTokenizer, Token, TokenIterator};

/*

- need to output xml for the parse tree, buy maybe I should put it into
a tree as well - in fact I want to do that for my own practice / something
I want to do.



// parsing logic


- basically how we are going to do this is 1 function for just about
1 non terminal rule in the grammar - this is acutally the same thing that is
done in crafting interpreters


LL grammar: can be parsed by a recursive descent parser without backtracking
- this is the case with jack - idk about other grammars though





basically for the jack implementation make a function for each non-terminal rule then just call
that rule to make the tree - from here I can have print_xml method for the tree which will print
it in XML. Not sure how different languages will implement these things. Rest of this is just
looking at the jack grammar

lexical elements
- keywords
- symbols
- integerConstant
- StringConstant
- identifier

theses are all the tokens

program structure

A jack program is a collection of classes, each appearing in a separate file, and each compiled separately.
each class is structured as follows

class: 'class' className '{' classVarDec* subroutineDec* '}'
classVarDec: ('static'|'field') type varName (','varName)* ';'
type: 'int'|'char'|'boolean'|className
subroutineDec: ('constructor'|'function'|'method') ('void'|type) subroutineName '(' parameterList ')' subroutineBody

.. more

Statements
A jack program includes statements, as follows

statements: statement*
statement: letStatement | ifStatement| whileStatement | doStatement| returnStatement

Expressions
A jack program includes expressions, as follows

expression: term (op term)*

*/

/*
- I want to have a in memory tree that I use - and I want to build it and derive the xml from that
I was thinking I would have nodes that implement a trait called ProgramNode and have them inherit that
and just put all of those in the tree, and from there figure shit out??? More importantly just do something
and see what happens
 */

/*
TODOS:
1. test this thing is actually correct
- good way to test small things is to write tests with small test programs or rules
- use the repl
2. refactor - there's a lot of things in here that are repeats or could be
make better. Here are some things off the top of my head I noticed
- use creational design pattern for node - I create nodes pretty often with some kinda
random stuff in them - I think I might be able to clean that up  (is builder good for these nodes?)

 */

pub trait ProgramNode: Debug {
    fn get_children(&self) -> &Option<Vec<Box<dyn ProgramNode>>>;
    fn get_children_mut(&mut self) -> &mut Option<Vec<Box<dyn ProgramNode>>>;
    fn set_empty_children(&mut self, children: Option<Vec<Box<dyn ProgramNode>>>);
    fn push(&mut self, child: Box<dyn ProgramNode>) {
        // println!("pushing -> {:#?}\n to {:#?}", child, self);
        match self.get_children_mut() {
            Some(children) => children.push(child),
            None => self.set_empty_children(Some(vec![child])), // None => self.children = Some(vec![child]),
        }
    } // should be void? or boolean to indicate it added succesfully?
    fn get_node_type(&self) -> &str;
    fn get_children_owned(self) -> Option<Vec<Box<dyn ProgramNode>>>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/*
had to make T static? Wtf? I mean it doesn't need to live the whole life of the program
and honeslty the lifetime here won't make it live the whole life of the program infact it'll
just make the compile stop complaing - T only needs to live as long as Node?
 */
#[derive(Debug)]
struct Node<T: Clone + 'static> {
    val: Option<T>,
    node_type: String, // right now a string, but it should really be an enum
    children: Option<Vec<Box<dyn ProgramNode>>>,
}

// should complain about not having methods implemented
impl<T: Debug + Clone> ProgramNode for Node<T> {
    // somehow need to get a value from here and also if other types
    // also need to implement this if I'm going to be able to write it for xml, but
    // I mean it shouldn't nec. be mando?
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self 
    }

    fn get_node_type(&self) -> &str {
        &self.node_type
    }
    fn get_children(&self) -> &Option<Vec<Box<dyn ProgramNode>>> {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Option<Vec<Box<dyn ProgramNode>>> {
        &mut self.children
    }
    fn set_empty_children(&mut self, children: Option<Vec<Box<dyn ProgramNode>>>) {
        self.children = children;
    }

    fn get_children_owned(self) -> Option<Vec<Box<dyn ProgramNode>>> {
        self.children
    }
}

impl<T: Clone> Node<T> {
    fn new(
        val: Option<T>,
        node_type: impl Into<String>,
        children: Option<Vec<Box<dyn ProgramNode>>>,
    ) -> Self {
        Node {
            val,
            node_type: node_type.into(),
            children: children,
        }
    }

    fn set_node_type(&mut self, s: impl Into<String>) {
        self.node_type = s.into();
    }

    fn make_box_empty_node_type(r#type: impl Into<String>) -> Box<Node<T>> {
        Box::new(Node {
            val: None,
            node_type: r#type.into(),
            children: None,
        })
    }

    fn push(&mut self, child: Box<dyn ProgramNode>) {
        match self.children.as_mut() {
            Some(children) => children.push(child),
            None => self.children = Some(vec![child]),
        }
    }
}

impl<T: Clone> Default for Node<T> {
    fn default() -> Self {
        Self {
            children: None,
            val: None,
            node_type: "I don't know what to put here".to_string(),
        }
    }
}

impl From<Token> for Node<Token> {
    fn from(tok: Token) -> Self {
        let r#type = match tok.clone() {
            Token::Identifier(_) => "identifier",
            Token::IntegerConstant(_) => "integerConstant",
            Token::Keyword(_) => "keyword",
            Token::StringConstant(_) => "stringConstant",
            Token::Symbol(_) => "symbol",
        }
        .to_string();
        Node::new(Some(tok), r#type, None)
    }
}

impl From<Token> for Box<Node<Token>> {
    fn from(token: Token) -> Self {
        Box::new(token.into())
    }
}

fn make_node(tok: Token, node_type: impl Into<String>) -> Node<Token> {
    let mut tok: Node<Token> = tok.into();
    tok.node_type = node_type.into();
    tok
}

fn make_box_node(tok: Token, node_type: impl Into<String>) -> Box<Node<Token>> {
    Box::new(make_node(tok, node_type))
}

pub struct AST {
    tokens: Peekable<TokenIterator>,
    root: Box<dyn ProgramNode>,
}

impl AST {
    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<&Token> {
        // this returns Option<&Token> I might need a Option<Token> - how do I do that?
        self.tokens.peek()
    }

    fn new(tokens: Vec<Token>) -> Self {
        AST {
            tokens: TokenIterator::from(tokens).peekable(),
            root: Box::new(Node::<Token>::default()),
        }
    }

    fn to_xml_jack_format_string(f: &mut File, root: &dyn ProgramNode) -> Result<()> {
        // I mean probably shouldn't need to create a string but I'm going to

        // let root = root
        //     .get_children_owned()
        //     .expect("thing")
        //     .get(0)
        //     .expect("class node");

        // basically expecting root === Node<Token> i.e. root.val == token
        let children = root.get_children().as_ref();

        let root_type = root.get_node_type(); // class, keyword, whatever

        // println!("current node val:{:#?}", root);

        Ok(match children {
            Some(children) => {
                f.write_all(format!("<{}>\n", root_type).as_bytes())?;
                for child in children {
                    Self::to_xml_jack_format_string(f, child.as_ref())?;
                }
                f.write_all(format!("</{}>", root_type).as_bytes())?;
            }
            None => {
                // go from &dyn ProgramNode -> &Node<Token>
                let node = root.as_any().downcast_ref::<Node<Token>>();
                if let Some(node_val) = node {
                    let node_val = node_val.val.clone();
                    if let Some(tok) = node_val {
                        // note: as_xml_string() includes newlines - which idk if it should or not
                        f.write_all(format!("{}", tok.as_xml_string()).as_bytes())?;
                    } else {
                        f.write_all(format!("<{}>\n</{}>\n", root_type, root_type).as_bytes())?;
                    }
                } else {
                    println!("node, but nothing in it");
                    dbg!(node);
                    // no node not no code
                    f.write_all(format!("<{}>no node?</{}>", root_type, root_type).as_bytes())?;
                }
            }
        })
    }

    // I feel like this should be possible to do with a reference, but
    // I'm going to end up consuming the object because that's easier
    // TODO: need to actually make my nodes match the grammar so I can easily out put
    // xml

    #[allow(dead_code)]
    pub fn create_xml_file_from_ast(self, write_path: PathBuf) -> Result<()> {
        let mut f = File::create(write_path)?;

        // have to define the type here, because this conversion doesn't
        // happen automatically, which is annoying
        let root_boxed: Box<dyn ProgramNode> = self.root;
        Self::to_xml_jack_format_string(&mut f, root_boxed.as_ref())
    }

    // top level parsing?
    fn parse(&mut self) {
        // if I want to return something from the AST I should then consume self
        // otherwise just mutate self (which is what I'm doing now)
        let all_nodes = self.parse_class();
        self.root = all_nodes;
    }

    // Program Structure

    fn parse_class(&mut self) -> Box<dyn ProgramNode> {
        let mut class: Box<Node<Token>> = self.next().expect("a type").into();
        class.set_node_type("keyword");
        let mut class_node = Node::<Token>::make_box_empty_node_type("class");
        class_node.push(class);

        class_node.push(self.parse_class_name());

        let mut open_bracket: Box<Node<Token>> = self.next().expect("a type").into();
        open_bracket.node_type = "symbol".into();

        class_node.push(open_bracket);

        while let "static" | "field" = self
            .peek()
            .expect("possible classVarDec")
            .get_value()
            .as_str()
        {
            class_node.push(self.parse_class_var_dec())
        }

        while let "constructor" | "function" | "method" = self
            .peek()
            .expect("possible classVarDec")
            .get_value()
            .as_str()
        {
            class_node.push(self.parse_subroutine_dec())
        }

        let close_bracket: Box<Node<Token>> = self.next().expect("a type").into();
        class_node.push(close_bracket);

        class_node
    }

    fn parse_class_var_dec(&mut self) -> Box<dyn ProgramNode> {
        let static_or_field: Box<Node<Token>> = self.next().expect("a type").into();
        let mut class_var_dec_node = Node::<Token>::make_box_empty_node_type("classVarDec");
        class_var_dec_node.push(static_or_field);

        class_var_dec_node.push(self.parse_type());
        class_var_dec_node.push(self.parse_var_name());

        while let "," = self.peek().expect("possible comma").get_value().as_str() {
            let comma: Box<Node<Token>> = self.next().expect("comma").into();
            class_var_dec_node.push(comma);
            class_var_dec_node.push(self.parse_var_name());
        }

        let semi_colon: Box<Node<Token>> = self.next().expect("semi colon").into();
        class_var_dec_node.push(semi_colon);

        class_var_dec_node
    }

    fn parse_type(&mut self) -> Box<dyn ProgramNode> {
        let r#type = self.next().expect("a type");
        let mut type_node = Node::<Token>::make_box_empty_node_type("type");

        // pretty sure this is irrelavent..
        // if let "int" | "char" | "boolean" = r#type.get_value().as_str() {
        //     let primitive_type: Box<Node<Token>> = r#type.into();
        //     type_node.push(primitive_type);
        // } else {
        //     type_node.push(self.parse_class_name());
        // }
        let boxed_type: Box<Node<Token>> = r#type.into();
        type_node.push(boxed_type);

        // wtf happens here if children is None?
        let Node { children , .. } = *type_node;
        let mut children = children.expect("children to not be None");
        children.swap_remove(0)
    }

    fn parse_subroutine_dec(&mut self) -> Box<dyn ProgramNode> {
        let subroutine_keyword_type: Box<Node<Token>> = self.next().expect("a type").into();
        let mut subroutine_dec_node = Node::<Token>::make_box_empty_node_type("subroutineDec");
        subroutine_dec_node.push(subroutine_keyword_type);

        if let "void" = self.peek().expect("possible comma").get_value().as_str() {
            let void_keyword: Box<Node<Token>> = self.next().expect("void keyword").into();
            subroutine_dec_node.push(void_keyword);
        } else {
            subroutine_dec_node.push(self.parse_type());
        }
        subroutine_dec_node.push(self.parse_subroutine_name());

        let open_paren: Box<Node<Token>> = self.next().expect("a type").into();
        subroutine_dec_node.push(open_paren);

        let parameter_list = self.parse_parameter_list();
        // subroutine_dec_node.push(self.parse_parameter_list());
        subroutine_dec_node.push(parameter_list);

        let close_paren: Box<Node<Token>> = self.next().expect("a type").into();
        subroutine_dec_node.push(close_paren);

        subroutine_dec_node.push(self.parse_subroutine_body());

        subroutine_dec_node
    }

    // TODO: JUST THESE LAST 2 THEN DONE - AFTER THAT IT'S OFF TO TESTING
    fn parse_parameter_list(&mut self) -> Box<dyn ProgramNode> {
        // parameterList: ((type varName) (',' type varName)*)? <- this is the rule for this
        // I've never seen something wrapped in parens in this notation and I don't 100% what it
        // means, but if an issue happens I'll come back to it

        let mut parameter_list_node = Node::<Token>::make_box_empty_node_type("parameterList");
        if let ")" = self.peek().expect("").get_value().as_str() {
            return parameter_list_node;
        }

        parameter_list_node.push(self.parse_type());
        parameter_list_node.push(self.parse_var_name());
        while let "," = self
            .peek()
            .expect("some kind of token")
            .get_value()
            .as_str()
        {
            let comma: Box<Node<Token>> = self.next().expect("").into();
            parameter_list_node.push(comma);
            parameter_list_node.push(self.parse_type());
            parameter_list_node.push(self.parse_var_name());
        }

        parameter_list_node
    }

    fn parse_subroutine_body(&mut self) -> Box<dyn ProgramNode> {
        let mut subroutine_body_node = Node::<Token>::make_box_empty_node_type("subroutineBody");
        let open_bracket: Box<Node<Token>> = self.next().expect("open bracket").into();
        subroutine_body_node.push(open_bracket);

        while let "var" = self.peek().expect("closing comma").get_value().as_str() {
            subroutine_body_node.push(self.parse_var_dec());
        }
        let statements = self.parse_statements();
        if statements.is_some() {
            subroutine_body_node.push(statements.unwrap());
        }

        let close_bracket: Box<Node<Token>> = self.next().expect("close bracket").into();
        subroutine_body_node.push(close_bracket);

        subroutine_body_node
    }

    fn parse_var_dec(&mut self) -> Box<dyn ProgramNode> {
        let mut var_dec_node = Node::<Token>::make_box_empty_node_type("varDec");

        let var_keyword: Box<Node<Token>> = self.next().expect("var keyword").into();
        var_dec_node.push(var_keyword);

        let r#type: Box<Node<Token>> = self.next().expect("a type").into();
        var_dec_node.push(r#type);
        let var_name = self.parse_var_name();
        var_dec_node.push(var_name);

        let comma = self.peek().expect("closing comma").get_value();
        while "," == comma.as_str() {
            let comma: Box<Node<Token>> = self.next().expect("comma").into();
            var_dec_node.push(comma);
            let var_name = self.parse_var_name();
            var_dec_node.push(var_name);
        }

        let semi_colon: Box<Node<Token>> = self.next().expect("semi colon").into();
        var_dec_node.push(semi_colon);
        var_dec_node
    }

    fn parse_class_name(&mut self) -> Box<dyn ProgramNode> {
        // let mut class_name_node: Box<Node<Token>> = Node::new(None, "class_name_node", None).into();
        // let class_name_ident: Box<Node<Token>> = self.next().expect("class_name").into();
        // class_name_node.push(class_name_ident);
        // class_name_node
        let class_name_ident: Box<Node<Token>> = self.next().expect("class_name").into();
        class_name_ident
    }

    fn parse_subroutine_name(&mut self) -> Box<dyn ProgramNode> {
        // let mut subroutine_name_node: Box<Node<Token>> =
        //     Node::new(None, "subroutine_name_node", None).into();
        // let mut subroutine_name: Box<Node<Token>> = self.next().expect("subroutine name").into();
        // subroutine_name.node_type = "identifier".into();
        // subroutine_name_node.push(subroutine_name);

        // subroutine_name_node
        let subroutine_name: Box<Node<Token>> = self.next().expect("subroutine name").into();
        subroutine_name
    }

    fn parse_var_name(&mut self) -> Box<dyn ProgramNode> {
        //let mut var_name: Box<Node<Token>> = Node::new(None, "var_name", None).into();
        let var_name: Box<Node<Token>> = self.next().expect("varName").into();
        var_name
    }

    // Statements

    // can I just return the statments node even if it's empty and not used?
    // I mean probs, but it'll make my tree have some useless information in it
    fn parse_statements(&mut self) -> Option<Box<dyn ProgramNode>> {
        // let next_statement = self.peek();
        let mut statements_node: Box<Node<Token>> = Node::new(None, "statements", None).into();

        while let Some(inner) = self.peek().map(|v| v.get_value()) {
            match inner.as_str() {
                "let" | "if" | "while" | "do" | "return" => {
                    // also I could just pass the keys to a map and have the map return
                    // a function that I call?
                    let statement = self.parse_statement();
                    statements_node.push(statement);
                    // if inner.as_str() == "do" || inner.as_str() == "return" {
                    //     let semi_colon: Box<Node<Token>> = self.next().expect("semi_colon").into();
                    //     statement.push(semi_colon);
                    // }
                    // statements_node.push(statement);
                }
                _ => return Some(statements_node),
            }
        }
        if statements_node.children.is_some() {
            return Some(statements_node); // need this because for some reason not returning properly
        }
        None
    }

    fn parse_statement(&mut self) -> Box<dyn ProgramNode> {
        let statement = self.peek().expect("a statement");
        match statement.get_value().as_str() {
            "let" => self.parse_let_statement(),
            "do" => self.parse_do_statment(),
            "if" => self.parse_if_statement(),
            "while" => self.parse_while_statement(),
            "return" => self.parse_return_statement(),
            _ => panic!("in statemetns but didn't match a statement"),
        }
    }

    fn parse_let_statement(&mut self) -> Box<dyn ProgramNode> {
        let let_statement: Box<Node<Token>> = self.next().expect("let keyword").into();
        let mut let_statement_node = Node::<Token>::make_box_empty_node_type("letStatement");
        let_statement_node.push(let_statement);

        // TODO: figure out how you do this??
        // let_statement.push(self.next().expect("varName")::<Box<Node<Token>>>into());
        let ident: Box<Node<Token>> = self.next().expect("varName").into();
        let_statement_node.push(ident);
        let square_brackt_or_equals = self.peek().expect("sqaure bracket or equals").get_value();

        if square_brackt_or_equals.as_str() == "[" {
                let open_bracket: Box<Node<Token>> = self.next().expect("open bracket").into();
                let_statement_node.push(open_bracket);
                let expr = self.parse_expression();
                let_statement_node.push(expr);
                let closing_bracket: Box<Node<Token>> =
                    self.next().expect("closing bracket").into();
                let_statement_node.push(closing_bracket);
                // let semi_colon: Box<Node<Token>> = self.next().expect("semi colon").into();
                // let_statement_node.push(semi_colon);
            }
                // should be = if not should panic / throw error
                let equals_sign: Box<Node<Token>> = self.next().expect("equals symbol").into();
                let_statement_node.push(equals_sign);
                let expr = self.parse_expression();
                let_statement_node.push(expr);
                let semi_colon: Box<Node<Token>> = self.next().expect("semi colon").into();
                let_statement_node.push(semi_colon);

        let_statement_node
    }

    // could be helper
    //     fn parse_open_bracket_statments_closing_bracket(&mut self)  {
    //
    //     }

    fn parse_if_statement(&mut self) -> Box<dyn ProgramNode> {
        let mut if_statement_node = Node::<Token>::make_box_empty_node_type("ifStatement");

        let if_keyword: Box<Node<Token>> = self.next().expect("if keyword").into();
        if_statement_node.push(if_keyword);

        let opening_comma: Box<Node<Token>> = self.next().expect("a comma").into();
        if_statement_node.push(opening_comma);
        let expr = self.parse_expression();
        if_statement_node.push(expr);
        // TODO: notice I'm not donig any checking that the data is actually a closing comma
        // only that it's a string - can I have a type that represents a closing openging/closing commas?
        let closing_comma: Box<Node<Token>> = self.next().expect("closing comma").into();
        if_statement_node.push(closing_comma);

        let opening_bracket: Box<Node<Token>> = self.next().expect("a comma").into();
        if_statement_node.push(opening_bracket);
        let statements = self.parse_statements();
        if let Some(statements) = statements {
            if_statement_node.push(statements);
        }
        let closing_bracket: Box<Node<Token>> = self.next().expect("a comma").into();
        if_statement_node.push(closing_bracket);

        // I should really need to get the value every time I should be able to match on the
        // &str inside - AsRef maybe?
        let r#else = self.peek().expect("else statements").get_value();

        if r#else.as_str() == "else" {
            // parse else statement
            let else_keyword: Box<Node<Token>> = self.next().expect("else keyword").into();
            if_statement_node.push(else_keyword);
            let opening_bracket: Box<Node<Token>> = self.next().expect("a comma").into();
            if_statement_node.push(opening_bracket);
            let statements = self.parse_statements();
            if let Some(statements) = statements {
                if_statement_node.push(statements);
            }
            let closing_bracket: Box<Node<Token>> = self.next().expect("a comma").into();
            if_statement_node.push(closing_bracket);
        }

        if_statement_node
    }

    fn parse_while_statement(&mut self) -> Box<dyn ProgramNode> {
        let mut while_statement_node = Node::<Token>::make_box_empty_node_type("whileStatement");

        println!("parse while statement");
        let while_statement_keyword: Box<Node<Token>> = self.next().expect("the word while").into();

        if while_statement_keyword.val != Some(Token::Keyword("while".to_string())) {
            panic!("{:?} is in the wrong place", while_statement_keyword.val);
        }

        while_statement_node.push(while_statement_keyword);

        let opening_paren: Box<Node<Token>> = self.next().expect("a comma").into();
        while_statement_node.push(opening_paren);
        let expr = self.parse_expression();
        while_statement_node.push(expr);
        // TODO: notice I'm not donig any checking that the data is actually a closing comma
        // only that it's a string - can I have a type that represents a closing openging/closing commas?
        let closing_paren: Box<Node<Token>> = self.next().expect("closing comma").into();
        while_statement_node.push(closing_paren);

        let opening_bracket: Box<Node<Token>> = self.next().expect("a comma").into();
        while_statement_node.push(opening_bracket);
        let statements = self.parse_statements();
        if let Some(statements) = statements {
            while_statement_node.push(statements);
        }
        let closing_bracket: Box<Node<Token>> = self.next().expect("a comma").into();
        while_statement_node.push(closing_bracket);

        while_statement_node
    }

    // TODO: need to test
    fn parse_do_statment(&mut self) -> Box<dyn ProgramNode> {
        let do_keyword: Box<Node<Token>> = self.next().expect("do keyword").into();
        let mut do_statment_node = Box::new(Node::<Token>::new(None, "doStatement", None));
        do_statment_node.push(do_keyword);

        // TODO: parse_subroutine_call not implemented in a function instead it's in term rn
        debug!("using self.parse_subroutine_call() - which is different from subroutine_call() handling in parse_term()");
        let mut sub_call = self.parse_subroutine_call();
        let children = sub_call.as_any_mut().downcast_mut::<Node<Token>>().expect("a Node<Token> - aborting program if it isn't").children.as_mut().expect("children as mut");
        // do_statment_node.push(sub_call);
        do_statment_node.children.as_mut().expect("children of do_statement_node").append(children);
        let semi_colon: Box<Node<Token>> = self.next().expect("semi-colon").into();
        do_statment_node.push(semi_colon);
        do_statment_node
    }

    // TODO: need to test
    fn parse_return_statement(&mut self) -> Box<dyn ProgramNode> {
        // way to valudate the data I' have rather than the type maybe something like
        // an attribute macro? like #[is("return")]
        let return_keyword = self.next().expect("return keyword");
        let mut ret_node = Box::new(Node::<String>::new(None, "returnStatement", None));
        ret_node.push(make_box_node(return_keyword, "keyword"));

        if Some(&Token::Symbol(";".into())) != self.peek() {
            let expr = self.parse_expression();
            ret_node.push(expr);
        }

        let mut semi_colon: Box<Node<Token>> = self.next().expect("semi colon").into();
        semi_colon.node_type = "symbol".into();
        ret_node.push(semi_colon);
        ret_node
    }

    // Expressions

    pub fn parse_expression(&mut self) -> Box<dyn ProgramNode> {
        // expression: term (op term)*

        // let term = self.next().expect("a term");

        println!("parsing expression");

        let mut expression: Box<dyn ProgramNode> =
            Box::new(Node::<String>::new(None, "expression", None));

        let term = self.parse_term();
        expression.push(term);

        while let Some(Token::Symbol(string)) = self.peek() {
            println!("looking at {:?}", string);
            if matches!(
                string.as_str(),
                "+" | "-" | "*" | "/" | "&" | "|" | "<" | ">" | "="
            ) {
                // these are the ops
                // don't need parse op method because this is a terminal
                let op = Box::new(Node::new(
                    Some(self.next().expect("an op")),
                    "op".to_string(),
                    None,
                ));
                expression.push(op);
                let next_term = self.parse_term();
                expression.push(next_term);
            } else {
                break; // no more expressions
            }
        }

        expression
    }

    // if using impl PorgramNode you can only return 1 type not multiple types
    // if returning multiple types (like in my case) I need to return a trait object
    fn parse_term(&mut self) -> Box<dyn ProgramNode> {
        // really I should start with a peek here then go from there
        // with just peek I can figure out if it's :
        //  integerConstant
        //  stringConstant
        //  keywordConstant
        //  and if it's not one of those it's either ->  varName,
        // varName[expression], subroutineCall | (expressiona) | unaryOp term
        //  OR just keep this the same and implement subroutineCall seperately from term?

        let term = self.next().expect(
            "this should be a term if not panic because I'm not going to handle errors here",
        );

        println!("parsing term: {:?}", term);

        // basically will have these empty nodes which represent non-terminal rules - and terminal hold values
        // TODO: think about this: an empty enum or struct would be better than a string here
        let mut term_node: Box<dyn ProgramNode> =
            Box::new(Node::<Token>::new(None, "term".to_string(), None));

        // type returned from match arsm have to be the same? but from function I'm thinking not
        match term.clone() {
            Token::IntegerConstant(_) | Token::StringConstant(_) => {
                // for now just returning a stinrg even though I don't really want to :(
                let term: Box<Node<Token>> = term.clone().into();
                term_node.push(term);
                term_node
            }
            Token::Keyword(keyword) => {
                if matches!(keyword.as_str(), "true" | "false" | "null" | "this") {
                    // keyword constant
                    let term: Box<Node<Token>> = term.clone().into();
                    term_node.push(term);
                    term_node
                } else {
                    // no sure what to do otherwiese
                    panic!(
                        "matched a keyword const in term, but it wasn't recognized: {}",
                        keyword
                    );
                }
            }
            Token::Symbol(symbol) => {
                // this has to be the '(' expression ')' OR term OR
                // unaryOp term

                let symb = symbol.as_str();
                if symb == "(" {
                    trace!("looking at a \"'(' expression ')'\"");
                    // at this point term == (
                    let open_paren = make_box_node(term, "symbol");
                    term_node.push(open_paren);
                    let expr = self.parse_expression(); // should return a node
                    println!("finished parsing expression");
                    term_node.push(expr);
                    let closing_paren = make_box_node(
                        self.next().expect("a clsoing paren for term non-term rule"),
                        "symbol",
                    );
                    term_node.push(closing_paren);
                    term_node
                } else if symb == "-" || symb == "~" {
                    trace!("looking at \"unaryOp term\"");
                    // at this point term == unaryOp i.e. '-' or '_'
                    let unary_op = make_box_node(term, "symbol");
                    term_node.push(unary_op);
                    let term = self.parse_term();
                    term_node.push(term);
                    term_node
                } else {
                    panic!(
                        "unknown symbol: '{:?}' found in '(' expression ')' non-terminal",
                        symbol
                    );
                }
            }
            Token::Identifier(_) => {
                // 1 of varName | varName '[' expression ']' | subroutineCall |

                let next = self.peek();
                println!("parse term: next token is: {:?}", next);
                //  match next { <- come back to this and see if you can fix with pattern matching

                match next {
                // match next.map(|v| v.clone()) {
                    None => panic!("panicing because we got none from peek - not exactly sure what to do here yet"),
                    Some(Token::Symbol(symbol)) => {
                        if symbol == "[" {
                            // basically the question is am I returning 1 node or a bunch of them?
                            // this is VarName '[' expression ']'
                            let var_name: Box<Node<Token>> = term.into();
                            term_node.push(var_name);
                            let open_bracket:Box<Node<Token>> = self.next().expect("open bracket").into();

                            term_node.push(open_bracket);

                            let expression = self.parse_expression(); // this needs to return a node so I can add it as a child to the term
                            term_node.push(expression);

                            let closing_bracket:Box<Node<Token>> = self.next().expect("closing bracket").into();
                            term_node.push(closing_bracket);
                            term_node
                        } else {
                            let subroutine_name_or_class_name_or_var_name =
                                make_box_node(term.clone(), "identifier");

                            // NOTE: in the reference implementation they give us for the parser we don't create a node
                            // for the subroutine rule - instead it just gets added to the term node - why idk - I'm
                            // sure if it's needed I can modify the tree to have this if I need it for something
                            // for _now going to comment it out_
                            // let mut sub_rout_node: Box<dyn ProgramNode> =
                            //     Box::new(Node::<String>::new(None, "subroutineCall", None));
                            // NOTE: this code is duplicated in parse_subroutine_call
                            // sub_rout_node.push(subroutine_name_or_class_name_or_var_name);
                            term_node.push(subroutine_name_or_class_name_or_var_name);
                            if symbol == "(" {
                                // this must be subroutineCall
                                // at this point we know term is the subroutineName
                                let opening_paren =
                                    make_box_node(self.next().expect("opening paren"), "symbol");
                                term_node.push(opening_paren);
                                // expression_list is optional so at this point I should first of all
                                // make sure that it is actually what I think it is

                                let expression_list_vals = self.parse_expression_list();
                                let closing_paren =
                                        make_box_node(self.next().expect("a )"), "symbol");

                                   term_node 
                                        .push(expression_list_vals);
                                    term_node.push(closing_paren);
                                    term_node
                            } else if symbol == "." {
                                // at this point term is className | varName
                                let dot = make_box_node(
                                    self.next().expect("dot in subroutineCall"),
                                    "symbol",
                                );
                                term_node.push(dot);
                                let subroutine_name = make_box_node(
                                    self.next().expect("subroutine name"),
                                    "identifier",
                                );
                                term_node.push(subroutine_name);
                                let left_paren =
                                    make_box_node(self.next().expect("left paren"), "symbol");
                                term_node.push(left_paren);
                                let expression_list = self.parse_expression_list();
                                term_node.push(expression_list);
                                let right_paren =
                                    make_box_node(self.next().expect("right paren"), "symbol");
                                term_node.push(right_paren);
                                // term_node.push(sub_rout_node);
                                term_node
                            } else {
                                // CHANGED: this should be varName 
                                // panic!(
                                //     "encountered a symbol that I don't know what it is: {}",
                                //     symbol
                                // );

                               //  let var_name: Box<Node<Token>> = term.into();
                               //  term_node.push(var_name);
                               // how do we not always put thinsg in here twice...
                                term_node
                            }
                        }
                    }
                    _ => panic!("catchall panic")
                }
            }
        }
    }

    fn parse_subroutine_call(&mut self) -> Box<dyn ProgramNode> {
        let mut sub_rout_node: Box<dyn ProgramNode> =
            Box::new(Node::<Token>::new(None, "subroutine_call", None));
        let subroutine_name_or_class_name_or_var_name = make_box_node(
            self.next().expect("subroutine name, className, or varName"),
            "identifier",
        );

        let symbol = self.peek().expect("( OR className OR varName").get_value();

        sub_rout_node.push(subroutine_name_or_class_name_or_var_name);
        if symbol == "(" {
            let opening_paren = make_box_node(self.next().expect("opening paren"), "symbol");
            sub_rout_node.push(opening_paren);

            let expression_list_vals = self.parse_expression_list();
            sub_rout_node.push(expression_list_vals);
            let closing_paren = make_box_node(self.next().expect("a )"), "symbol");
            sub_rout_node.push(closing_paren);

            sub_rout_node
        } else if symbol == "." {
            // at this point term is className | varName
            let dot = make_box_node(self.next().expect("dot in subroutineCall"), "symbol");
            sub_rout_node.push(dot);
            let subroutine_name =
                make_box_node(self.next().expect("subroutine name"), "identifier");
            sub_rout_node.push(subroutine_name);
            let left_paren = make_box_node(self.next().expect("left paren"), "symbol");
            sub_rout_node.push(left_paren);
            let expression_list = self.parse_expression_list();
            sub_rout_node.push(expression_list);
            let right_paren = make_box_node(self.next().expect("right paren"), "symbol");
            sub_rout_node.push(right_paren);
            sub_rout_node
        } else {
            panic!(
                "encountered a symbol that I don't know what it is: {}",
                symbol
            );
        }
    }
    fn parse_expression_list(&mut self) -> Box<dyn ProgramNode> {
        println!("parsing expression list");
        let optional_expression_list = self.peek().expect("expression list");

        // somehow I determined that if this is ) then it's not an expression list??
        // ohhh because then it's just a closing parent for the method/function call
        let mut expression_list: Box<Node<Token>> =
            Box::new(Node::new(None, "expressionList", None));
        if optional_expression_list.get_value().as_str() == ")" {
            return expression_list;
        }

        let expression = self.parse_expression();
        expression_list.push(expression);

        while let Some(Token::Symbol(comma)) = self.peek() {
            if comma.as_str() == "," {
                let comma =
                    make_box_node(self.next().expect("comman in expression list"), "symbol");
                expression_list.push(comma);
                let next_expression = self.parse_expression(); // this is obvi. a problem because I need to put the expression in the tree
                expression_list.push(next_expression);
            } else {
                break;
            }
        }

        println!("return of expression list: {:#?}", expression_list);
        expression_list
    }
}

pub fn main(jack_file_path: String) -> Result<()> {
    // needs to take in a file to read
    // then tokenize it (using the scanner)
    // then create a parse tree from that
    // that's what the parser should do

    // TODO: should probably make sure jack_file_path exists
    // before I start using it

    // From<String> for PathBuf -> Into<PathBuf> for String && implies From<String> for PathBuf | From<PathBuf> for String
    let tokens = JackTokenizer::new(jack_file_path.into())?;
    let tokens = tokens.run(Option::<String>::None)?;
    // let it = tokens.into_iter().peekable();

    // let token_iter = TokenIterator::from(tokens);

    // for tok in token_iter {
    // dbg!(tok);
    // }

    trace!("tokens read from jack-tokenizer {:#?}", &tokens);
    let mut ast = AST::new(tokens);

    let parsed_output = ast.parse();
    // debug!("parsed_output = {:#?}", parsed_output);

    let mut f = File::create("ast-output")?;
    // f.write_all(parsed_output.de)
    let output = format!("{:#?}", parsed_output);
    f.write_all(output.as_bytes())?;

    return Ok(());
}

pub fn repl() -> Result<()> {
    // read the input from stdin and then send it through the parser

    println!("Jack parser: V0.0.1");
    loop {
        print!("> ");
        let mut buf = String::new();
        stdin().read_line(&mut buf)?;

        if buf == "exit" {
            break;
        }

        let scan = scanner::JackTokenizer::new_empty();
        let toks = scan.run(Some(buf))?;

        // TODO: this isn't even being used
        let _ast = AST::new(toks);

        panic!("not sure how to parse the tokens, because it's not clear what I can get.... expressoin class etc.");
        // let parsed_output = ast.
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::path::{Path, PathBuf};

    // don't know exactly what this does and I should
    // but pretty sure it just imports the module above
    use super::*;

    // JackToknizer::tokenize("(1+2,3+4,5+6)") <- somehting like this would make things way better
    // it's kind of a pain in the ass to work with the current API

    /* HERLPER FUNCTIONS */
    fn tokenized_input() -> Vec<Token> {
        // no reason tokenizer can't take String input why only a file??
        let jt = JackTokenizer::new("testing/parser_testing/expression_list.jack".into())
            .expect("no issue creaing tokenizer");

        jt.run(Option::<String>::None).expect("tokens")
    }

    fn tokenize_input_string(s: String) -> Result<Vec<Token>> {
        JackTokenizer::new_empty().run(Some(s))
    }

    fn create_paths(base: impl AsRef<Path>, file: impl AsRef<Path>) -> [PathBuf; 2] {
        let mut root: PathBuf = "testing".into();
        root.push(base);
        let mut read_path = root.clone();
        let mut write_path = root.clone();
        // if file is main.jack I want main.jack.out
        let output_file: String = format!(
            "{}.my.xml",
            file.as_ref().to_str().expect("to go to a string")
        );
        read_path.push(file);
        write_path.push(output_file);
        [read_path, write_path]
    }



    #[test]
    fn test_parse_expression_list() {
        let toks = tokenized_input();

        // let v1 = vec![1, 2, 3];
        let v2 = vec![
            Token::Symbol("(".into()),
            Token::IntegerConstant(1),
            Token::Symbol("+".into()),
            Token::IntegerConstant(2),
            Token::Symbol(")".into()),
        ];

        assert_eq!(toks, v2);
    }

    #[test]
    fn test_parse_term() {
        assert_eq!(0, 0);
    }

    #[test]
    fn test_parse_test_expression() {
        let tokens = tokenized_input();

        let mut p = AST::new(tokens.clone());

        dbg!(tokens);
        // let parsed_output = p.parse_term();
        let parsed_output = p.parse_expression();

        /*
        I just want to check that t and expected_output are the same
         */

        dbg!(parsed_output);
        // dbg!(expected_output);

        // TODO: can't do pattern matching....
        // let l = matches!(
        //     *parsed_output,
        //     Node {
        //         children: None,
        //         node_type,
        //         val
        //     }
        // );
        // assert!(l);
    }

    fn ast_thing(toks: Vec<Token>) -> Box<dyn ProgramNode> {
        let mut new_ast = AST::new(toks);
        new_ast.parse_expression()
    }
    #[test]
    fn test_expression_again() -> Result<()> {
        // not sure how results work here
        let toks = tokenize_input_string("1 + 2 + 3".into())?;

        dbg!(&toks);

        let expression_tree = ast_thing(toks);
        dbg!(expression_tree);

        Ok(())
    }

    #[test]
    fn test_parse_statements() -> Result<()> {
        // not sure how results work here
        let toks = tokenize_input_string(
            r#"
        
        /* Prints some text using teh standard library. */
        do Output.printString("hello world");
        do Output.println();
        return;
        
        "#
            .into(),
        )?;

        dbg!(&toks);

        let mut new_ast = AST::new(toks);
        let expression_tree = new_ast.parse_statements().unwrap();
        dbg!(expression_tree);

        Ok(())
    }


    #[test]
    fn write_hello_jack_to_xml() -> Result<()> {
        let [readpath, write_path] = create_paths("personal_testing", "hello.jack");
        assert_eq!(true, readpath.is_file());

        let jt = JackTokenizer::new(readpath)?;
        let toks = jt.run(Option::<String>::None)?;
        dbg!(&toks);

        let mut new_ast = AST::new(toks);
        // parse tells me nothing...
        new_ast.parse();

        dbg!(&new_ast.root);

        new_ast.create_xml_file_from_ast(write_path)?;

        Ok(())
    }

    #[test]
    fn write_main_square_jack_to_xml() -> Result<()> {
        // PASSED: text compared with the reference they've given works 👍 
        let [readpath, write_path] = create_paths("Square", "Main.jack");
        assert_eq!(true, readpath.is_file());
        let jt = JackTokenizer::new(readpath)?;
        let toks = jt.run(Option::<String>::None)?;
        dbg!(&toks);

        let mut new_ast = AST::new(toks);
        // parse tells me nothing...
        new_ast.parse();

        dbg!(&new_ast.root);

        new_ast.create_xml_file_from_ast(write_path)?;

        Ok(())
    }

    #[test]
    fn write_square_game_to_xml() -> Result<()> {
        // PASSED: text compared with the reference they've given works 👍 
        let [readpath, write_path] = create_paths("Square", "SquareGame.jack");
        assert_eq!(true, readpath.is_file());

        let jt = JackTokenizer::new(readpath)?;
        let toks = jt.run(Option::<String>::None)?;
        dbg!(&toks);

        let mut new_ast = AST::new(toks);
        // parse tells me nothing...
        new_ast.parse();

        // dbg!(&new_ast.root);

        new_ast.create_xml_file_from_ast(write_path)?;

        Ok(())
    }

    #[test]
    fn write_square_jack_to_xml() -> Result<()> {
        // PASSED: text compared with the reference they've given works 👍 
        let [readpath, write_path] = create_paths("Square", "Square.jack");
        assert_eq!(true, readpath.is_file());

        let jt = JackTokenizer::new(readpath)?;
        let toks = jt.run(Option::<String>::None)?;
        dbg!(&toks);

        let mut new_ast = AST::new(toks);
        // parse tells me nothing...
        new_ast.parse();

        dbg!(&new_ast.root);

        new_ast.create_xml_file_from_ast(write_path)?;

        Ok(())
    }


    #[test]
    fn write_array_test_to_xml() -> Result<()> {
        // PASSED: text compared with the reference they've given works 👍 
        let [readpath, write_path] = create_paths("ArrayTest", "Main.jack");
        assert_eq!(true, readpath.is_file());

        let jt = JackTokenizer::new(readpath)?;
        let toks = jt.run(Option::<String>::None)?;
        dbg!(&toks);

        let mut new_ast = AST::new(toks);
        // parse tells me nothing...
        new_ast.parse();

        // dbg!(&new_ast.root);

        new_ast.create_xml_file_from_ast(write_path)?;

        Ok(())
    }


    #[test]
    fn write_expression_less_sqaure_main_to_xml() -> Result<()> {
        // PASSED: text compared with the reference they've given works 👍 
        let [readpath, write_path] = create_paths("ExpressionLessSquare", "Main.jack");
        assert_eq!(true, readpath.is_file());

        let jt = JackTokenizer::new(readpath)?;
        let toks = jt.run(Option::<String>::None)?;
        dbg!(&toks);

        let mut new_ast = AST::new(toks);
        // parse tells me nothing...
        new_ast.parse();

        // dbg!(&new_ast.root);

        new_ast.create_xml_file_from_ast(write_path)?;

        Ok(())
    }


    #[test]
    fn write_expression_less_sqaure_sqaure_to_xml() -> Result<()> {
        // PASSED: text compared with the reference they've given works 👍 
        let [readpath, write_path] = create_paths("ExpressionLessSquare", "Square.jack");
        assert_eq!(true, readpath.is_file());

        let jt = JackTokenizer::new(readpath)?;
        let toks = jt.run(Option::<String>::None)?;
        dbg!(&toks);

        let mut new_ast = AST::new(toks);
        // parse tells me nothing...
        new_ast.parse();

        // dbg!(&new_ast.root);

        new_ast.create_xml_file_from_ast(write_path)?;

        Ok(())
    }

    #[test]
    fn write_expression_less_sqaure_sqaure_game_to_xml() -> Result<()> {
        // PASSED: text compared with the reference they've given works 👍 
        let [readpath, write_path] = create_paths("ExpressionLessSquare", "SquareGame.jack");
        assert_eq!(true, readpath.is_file());

        let jt = JackTokenizer::new(readpath)?;
        let toks = jt.run(Option::<String>::None)?;
        dbg!(&toks);

        let mut new_ast = AST::new(toks);
        // parse tells me nothing...
        new_ast.parse();

        // dbg!(&new_ast.root);

        new_ast.create_xml_file_from_ast(write_path)?;

        Ok(())
    }
}
