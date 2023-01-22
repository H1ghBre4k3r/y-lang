use super::Rule;
use log::error;
use pest::iterators::Pair;

// TODO: Give each AstNode a position
type Position = (usize, usize);

#[derive(Debug, Clone)]
pub enum AstNode {
    If {
        condition: Box<AstNode>,
        if_block: Box<AstNode>,
        else_block: Option<Box<AstNode>>,
        position: Position,
    },
    Declaration {
        ident: Box<AstNode>,
        value: Box<AstNode>,
        position: Position,
    },
    Assignment {
        ident: Box<AstNode>,
        value: Box<AstNode>,
        position: Position,
    },
    Block {
        block: Vec<AstNode>,

        position: Position,
    },
    BinaryOp {
        verb: BinaryVerb,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
        position: Position,
    },
    Integer {
        value: i64,
        position: Position,
    },
    Str {
        value: String,
        position: Position,
    },
    Ident {
        value: String,
        position: Position,
    },
    FnCall {
        ident: Box<AstNode>,
        params: Vec<AstNode>,
        position: Position,
    },
    FnDef {
        params: Vec<AstNode>,
        type_annotation: Box<AstNode>,
        block: Box<AstNode>,
        position: Position,
    },
    Param {
        ident: Box<AstNode>,
        type_annotation: Box<AstNode>,
        position: Position,
    },
    TypeAnnotation {
        value: String,
        position: Position,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryVerb {
    GreaterThan,
    LessThan,
    Equal,
    Plus,
    Minus,
    Times,
}

impl AstNode {
    pub fn position(&self) -> Position {
        use AstNode::*;
        match self {
            If { position, .. }
            | Declaration { position, .. }
            | Assignment { position, .. }
            | Block { position, .. }
            | BinaryOp { position, .. }
            | Integer { position, .. }
            | Str { position, .. }
            | Ident { position, .. }
            | FnCall { position, .. }
            | FnDef { position, .. }
            | Param { position, .. }
            | TypeAnnotation { position, .. } => position.clone(),
        }
    }

    fn from_string(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::string);
        AstNode::Str {
            value: pair
                .clone()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .to_owned(),
            position: pair.line_col(),
        }
    }

    fn from_integer(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::integer);
        AstNode::Integer {
            value: pair.as_str().parse::<i64>().unwrap(),
            position: pair.line_col(),
        }
    }

    fn from_ident(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::ident);
        AstNode::Ident {
            value: pair.as_str().to_owned(),
            position: pair.line_col(),
        }
    }

    fn from_expression(pair: Pair<Rule>) -> AstNode {
        match pair.as_rule() {
            Rule::integer => Self::from_integer(pair),
            Rule::ident => Self::from_ident(pair),
            Rule::fnCall => Self::from_fn_call(pair),
            Rule::string => Self::from_string(pair),
            Rule::binaryExpr => Self::from_binary_expression(pair),
            Rule::fnDef => Self::from_fn_def(pair),
            _ => {
                error!(
                    "Unexpected expression '{}' at {}:{}",
                    pair.as_str(),
                    pair.line_col().0,
                    pair.line_col().1
                );
                std::process::exit(-1)
            }
        }
    }

    fn from_binary_expression(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::binaryExpr);

        let mut inner = pair.clone().into_inner();

        let lhs = Self::from_expression(inner.next().unwrap());

        let verb = inner.next().expect(&format!(
            "Expected verb in binary expression '{}' at {}:{}",
            pair.as_str(),
            pair.line_col().0,
            pair.line_col().1,
        ));

        let verb = match verb.as_str() {
            ">" => BinaryVerb::GreaterThan,
            "<" => BinaryVerb::LessThan,
            "==" => BinaryVerb::Equal,
            "+" => BinaryVerb::Plus,
            "-" => BinaryVerb::Minus,
            "*" => BinaryVerb::Times,
            _ => {
                error!(
                    "Unexpected binary verb '{}' at {}:{}",
                    verb.as_str(),
                    verb.line_col().0,
                    verb.line_col().1
                );
                std::process::exit(-1);
            }
        };

        let rhs = Self::from_expression(inner.next().unwrap());

        AstNode::BinaryOp {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            verb,
            position: pair.line_col(),
        }
    }

    fn from_fn_def(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::fnDef);

        let position = pair.line_col();

        let mut inner = pair.into_inner();

        let Some(param_list) = inner.next() else {
            error!("Expected param list in function definition at {}:{}", position.0, position.1);
            std::process::exit(-1);
        };
        let param_list = Self::from_param_list(param_list);

        let Some(type_annotation) = inner.next() else {
            error!("Expected return type annotation in function definition at {}:{}", position.0, position.1);
            std::process::exit(-1);
        };
        let type_annotation = Self::from_type_annotation(type_annotation);

        let Some(block) = inner.next() else {
            error!("Expected block in function definition at {}:{}", position.0, position.1);
            std::process::exit(-1);
        };
        let block = Self::from_block(block);

        AstNode::FnDef {
            params: param_list,
            type_annotation: Box::new(type_annotation),
            block: Box::new(block),
            position,
        }
    }

    fn from_param_list(pair: Pair<Rule>) -> Vec<AstNode> {
        assert_eq!(pair.as_rule(), Rule::paramList);

        let param_pairs = pair.into_inner();

        let mut params = vec![];

        for param in param_pairs {
            params.push(Self::from_param(param));
        }

        params
    }

    fn from_param(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::parameter);

        let position = pair.line_col();

        let mut inner = pair.into_inner();

        let ident = inner.next().unwrap();
        let ident = Self::from_ident(ident);

        let type_annotation = inner.next().unwrap();
        let type_annotation = Self::from_type_annotation(type_annotation);

        AstNode::Param {
            ident: Box::new(ident),
            type_annotation: Box::new(type_annotation),
            position,
        }
    }

    fn from_type_annotation(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::typeAnnotation);

        let position = pair.line_col();

        let mut inner = pair.into_inner();

        let type_pair = inner.next().unwrap();
        assert_eq!(type_pair.as_rule(), Rule::typeName);

        let type_name = type_pair.as_str();

        AstNode::TypeAnnotation {
            value: type_name.to_owned(),
            position,
        }
    }

    fn from_fn_call(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::fnCall);

        let position = pair.line_col();

        let mut inner = pair.into_inner();

        let ident = inner.next().unwrap();

        let mut params = vec![];

        for param in inner {
            let position = param.line_col();
            match param.as_rule() {
                Rule::integer => params.push(Self::from_integer(param)),
                Rule::ident => params.push(Self::from_ident(param)),
                Rule::string => params.push(Self::from_string(param)),
                Rule::fnCall => params.push(Self::from_fn_call(param)),
                _ => {
                    error!(
                        "Unexpected paramenter '{:?}' at {}:{}",
                        param.as_str(),
                        position.0,
                        position.1
                    );
                    std::process::exit(-1);
                }
            }
        }

        AstNode::FnCall {
            ident: Box::new(Self::from_ident(ident)),
            params,
            position,
        }
    }

    fn from_declaration(pair: Pair<Rule>) -> AstNode {
        let mut inner = pair.clone().into_inner();

        let ident = Self::from_ident(inner.next().expect(&format!(
            "Expected lvalue in declaration '{}' at {}:{}",
            pair.as_str(),
            pair.line_col().0,
            pair.line_col().1
        )));

        let value = inner.next().expect(&format!(
            "Expected rvalue in declaration '{}' at {}:{}",
            pair.as_str(),
            pair.line_col().0,
            pair.line_col().1
        ));
        let value = Self::from_expression(value);

        AstNode::Declaration {
            ident: Box::new(ident),
            value: Box::new(value),
            position: pair.line_col(),
        }
    }

    fn from_assignment(pair: Pair<Rule>) -> AstNode {
        let mut inner = pair.clone().into_inner();

        let ident = Self::from_ident(inner.next().expect(&format!(
            "Expected lvalue in assignment '{}' at {}:{}",
            pair.as_str(),
            pair.line_col().0,
            pair.line_col().1
        )));

        let value = inner.next().expect(&format!(
            "Expected rvalue in assignment '{}' at {}:{}",
            pair.as_str(),
            pair.line_col().0,
            pair.line_col().1
        ));
        let value = Self::from_expression(value);

        AstNode::Assignment {
            ident: Box::new(ident),
            value: Box::new(value),
            position: pair.line_col(),
        }
    }

    fn from_if(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::ifStmt);

        let position = pair.line_col();

        let mut inner = pair.into_inner();
        let condition = Self::from_expression(inner.next().unwrap());
        let if_block = inner.next().unwrap();
        let else_block = inner.next().map(|block| Box::new(Self::from_block(block)));

        AstNode::If {
            condition: Box::new(condition),
            if_block: Box::new(Self::from_block(if_block)),
            else_block,
            position,
        }
    }

    fn from_block(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::block);

        let position = pair.line_col();

        let block = pair.into_inner();

        let mut block_ast = vec![];

        for statement in block {
            block_ast.push(Self::from_statement(statement));
        }

        AstNode::Block {
            block: block_ast,
            position,
        }
    }

    pub fn from_statement(pair: Pair<Rule>) -> AstNode {
        match pair.as_rule() {
            Rule::ifStmt => Self::from_if(pair),
            Rule::fnCall => Self::from_fn_call(pair),
            Rule::declaration => Self::from_declaration(pair),
            Rule::assignment => Self::from_assignment(pair),
            _ => {
                error!(
                    "Unexpected statement '{}' at {}:{}",
                    pair.as_str(),
                    pair.line_col().0,
                    pair.line_col().1
                );
                std::process::exit(-1);
            }
        }
    }
}
