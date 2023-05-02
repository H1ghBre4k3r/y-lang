use pest::iterators::Pair;

use super::{Integer, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Literal(String),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    ArraySlice(Box<Type>),
    TupleArray {
        item_type: Box<Type>,
        size: Integer<()>,
    },
    Reference(Box<Type>),
}

impl Type {
    pub fn from_pair(pair: Pair<Rule>) -> Type {
        match pair.as_rule() {
            Rule::fnType => {
                let mut inner = pair.into_inner().peekable();

                let mut params = vec![];

                while let Some(param) = inner.next() {
                    if inner.peek().is_some() {
                        params.push(Type::from_pair(param));
                    } else {
                        return Type::Function {
                            params,
                            return_type: Box::new(Type::from_pair(param)),
                        };
                    }
                }
                unreachable!();
            }
            Rule::typeName => {
                let type_name = pair.as_str();
                Self::Literal(type_name.to_owned())
            }
            Rule::arraySlice => {
                let mut inner = pair.into_inner();

                let type_name = inner.next().unwrap();
                let type_name = Type::from_pair(type_name);
                Self::ArraySlice(Box::new(type_name))
            }
            Rule::tupleArray => {
                let mut inner = pair.into_inner();

                let item_type = inner.next().unwrap();
                let item_type = Type::from_pair(item_type);

                let size = inner.next().unwrap();
                let size = Integer::from_pair(size, "");

                Self::TupleArray {
                    item_type: Box::new(item_type),
                    size,
                }
            }
            Rule::reference => {
                let mut inner = pair.into_inner();

                let item_type = inner.next().unwrap();
                let item_type = Type::from_pair(item_type);

                Self::Reference(Box::new(item_type))
            }
            _ => unreachable!(),
        }
    }
}
