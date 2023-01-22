use super::Rule;
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
            _ => unreachable!(
                "Unexpected term '{}' at {}:{}",
                pair.as_str(),
                pair.line_col().0,
                pair.line_col().1
            ),
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
            _ => unreachable!(
                "Unexpected binary verb '{}' at {}:{}",
                verb.as_str(),
                verb.line_col().0,
                verb.line_col().1
            ),
        };

        let rhs = Self::from_expression(inner.next().unwrap());

        AstNode::BinaryOp {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            verb,
            position: pair.line_col(),
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
                _ => unreachable!(
                    "Unexpected paramenter '{:?}' at {}:{}",
                    param.as_str(),
                    position.0,
                    position.1
                ),
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
            _ => unreachable!(
                "Unexpected statement '{}' at {}:{}",
                pair.as_str(),
                pair.line_col().0,
                pair.line_col().1
            ),
        }
    }
}
