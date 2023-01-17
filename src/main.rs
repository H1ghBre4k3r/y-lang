extern crate pest;
#[macro_use]
extern crate pest_derive;

use clap::Parser as CParser;

#[derive(CParser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long)]
    file: std::path::PathBuf,
}

use pest::Parser;

#[derive(Parser)]
#[grammar = "pesca.pest"]
pub struct PescaParser;

#[derive(Debug)]
pub enum AstNode {
    If {
        condition: Box<AstNode>,
        block: Box<AstNode>,
    },
    Assignment {
        ident: Box<AstNode>,
        value: Box<AstNode>,
    },
    Block(Vec<AstNode>),
    BinaryOp {
        verb: BinaryVerb,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },
    Integer(i64),
    Str(String),
    Ident(String),
    FnCall {
        ident: Box<AstNode>,
        params: Vec<AstNode>,
    },
}

#[derive(Debug)]
pub enum BinaryVerb {
    GreaterThan,
    LessThan,
    Equal,
}

fn build_ast_from_term(pair: pest::iterators::Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::integer => AstNode::Integer(pair.as_str().parse::<i64>().unwrap()),
        Rule::ident => AstNode::Ident(pair.as_str().to_owned()),
        _ => panic!("invalid term '{:?}'", pair),
    }
}

fn build_ast_from_binary_expression(pair: pest::iterators::Pair<Rule>) -> AstNode {
    assert_eq!(pair.as_rule(), Rule::binaryExpr);

    let mut inner = pair.into_inner();

    let lhs = build_ast_from_term(inner.next().unwrap());

    let verb = match inner.next().unwrap().as_str() {
        ">" => BinaryVerb::GreaterThan,
        "<" => BinaryVerb::LessThan,
        "==" => BinaryVerb::Equal,
        verb => panic!("Unexpected binary verb '{}'", verb),
    };

    let rhs = build_ast_from_term(inner.next().unwrap());

    AstNode::BinaryOp {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        verb,
    }
}

fn build_ast_from_fn_call(pair: pest::iterators::Pair<Rule>) -> AstNode {
    assert_eq!(pair.as_rule(), Rule::fnCall);

    let mut inner = pair.into_inner();

    let ident = inner.next().unwrap().as_str();

    let mut params = vec![];

    for param in inner {
        match param.as_rule() {
            Rule::integer => params.push(AstNode::Integer(param.as_str().parse::<i64>().unwrap())),
            Rule::ident => params.push(AstNode::Ident(param.as_str().to_owned())),
            Rule::string => params.push(AstNode::Str(param.as_str().to_owned())),
            _ => panic!("Unsupported paramenter '{:?}'", param.as_str()),
        }
    }

    AstNode::FnCall {
        ident: Box::new(AstNode::Ident(ident.to_owned())),
        params,
    }
}

fn build_ast_from_expression(pair: pest::iterators::Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::binaryExpr => build_ast_from_binary_expression(pair),
        Rule::fnCall => build_ast_from_fn_call(pair),
        _ => panic!("Invalid expression '{:?}'", pair.as_str()),
    }
}

fn build_ast_from_assignment(pair: pest::iterators::Pair<Rule>) -> AstNode {
    println!("{:#?}", pair);

    let mut inner = pair.into_inner();

    let ident = AstNode::Ident(
        inner
            .next()
            .expect("No valid identifier given!")
            .as_str()
            .to_owned(),
    );

    let value = inner.next().expect("No valid rvalue given!");

    todo!()
}

fn build_ast_from_if(pair: pest::iterators::Pair<Rule>) -> AstNode {
    assert_eq!(pair.as_rule(), Rule::ifStmt);

    let mut inner = pair.into_inner();
    let condition = build_ast_from_expression(inner.next().unwrap());
    let block = inner.next().unwrap().into_inner();

    let mut block_ast = vec![];

    for statement in block {
        block_ast.push(build_ast_from_statement(statement));
    }

    AstNode::If {
        condition: Box::new(condition),
        block: Box::new(AstNode::Block(block_ast)),
    }
}

fn build_ast_from_statement(pair: pest::iterators::Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::ifStmt => build_ast_from_if(pair),
        Rule::fnCall => build_ast_from_fn_call(pair),
        Rule::assignment => build_ast_from_assignment(pair),
        _ => panic!("not supported statement '{:?}'", pair.as_str()),
    }
}

fn main() {
    let args = Cli::parse();

    let file_content = std::fs::read_to_string(&args.file).expect(&format!(
        "Could not read file: '{}'",
        args.file.to_string_lossy()
    ));

    let pairs = PescaParser::parse(Rule::program, &file_content).expect("failed to parse file");

    let mut ast = vec![];

    for pair in pairs {
        if pair.as_rule() != Rule::EOI {
            ast.push(build_ast_from_statement(pair));
        }
    }

    println!("{:#?}", ast);
}
