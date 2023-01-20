use super::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
pub enum BinaryVerb {
    GreaterThan,
    LessThan,
    Equal,
}

impl AstNode {
    fn from_string(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::string);
        AstNode::Str(pair.as_str().to_owned())
    }

    fn from_integer(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::integer);
        AstNode::Integer(pair.as_str().parse::<i64>().unwrap())
    }

    fn from_ident(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::ident);
        AstNode::Ident(pair.as_str().to_owned())
    }

    fn from_expression(pair: Pair<Rule>) -> AstNode {
        match pair.as_rule() {
            Rule::integer => Self::from_integer(pair),
            Rule::ident => Self::from_ident(pair),
            Rule::fnCall => Self::from_fn_call(pair),
            Rule::string => Self::from_string(pair),
            Rule::binaryExpr => Self::from_binary_expression(pair),
            _ => unreachable!("invalid term '{:?}'", pair),
        }
    }

    fn from_binary_expression(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::binaryExpr);

        let mut inner = pair.into_inner();

        let lhs = Self::from_expression(inner.next().unwrap());

        let verb = match inner.next().unwrap().as_str() {
            ">" => BinaryVerb::GreaterThan,
            "<" => BinaryVerb::LessThan,
            "==" => BinaryVerb::Equal,
            verb => unreachable!("Unexpected binary verb '{}'", verb),
        };

        let rhs = Self::from_expression(inner.next().unwrap());

        AstNode::BinaryOp {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            verb,
        }
    }

    fn from_fn_call(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::fnCall);

        let mut inner = pair.into_inner();

        let ident = inner.next().unwrap();

        let mut params = vec![];

        for param in inner {
            match param.as_rule() {
                Rule::integer => params.push(Self::from_integer(param)),
                Rule::ident => params.push(Self::from_ident(param)),
                Rule::string => params.push(Self::from_string(param)),
                Rule::fnCall => params.push(Self::from_fn_call(param)),
                _ => unreachable!("Unsupported paramenter '{:?}'", param.as_str()),
            }
        }

        AstNode::FnCall {
            ident: Box::new(Self::from_ident(ident)),
            params,
        }
    }

    fn from_assignment(pair: Pair<Rule>) -> AstNode {
        let mut inner = pair.into_inner();

        let ident = Self::from_ident(inner.next().expect("No valid identifier given!"));

        let value = inner.next().expect("No valid rvalue given!");
        let value = Self::from_expression(value);

        AstNode::Assignment {
            ident: Box::new(ident),
            value: Box::new(value),
        }
    }

    fn from_if(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::ifStmt);

        let mut inner = pair.into_inner();
        let condition = Self::from_expression(inner.next().unwrap());
        let block = inner.next().unwrap();

        AstNode::If {
            condition: Box::new(condition),
            block: Box::new(Self::from_block(block)),
        }
    }

    fn from_block(pair: Pair<Rule>) -> AstNode {
        assert_eq!(pair.as_rule(), Rule::block);

        let block = pair.into_inner();

        let mut block_ast = vec![];

        for statement in block {
            block_ast.push(Self::from_statement(statement));
        }

        AstNode::Block(block_ast)
    }

    pub fn from_statement(pair: Pair<Rule>) -> AstNode {
        match pair.as_rule() {
            Rule::ifStmt => Self::from_if(pair),
            Rule::fnCall => Self::from_fn_call(pair),
            Rule::assignment => Self::from_assignment(pair),
            _ => unreachable!("not supported statement '{:?}'", pair.as_str()),
        }
    }
}
