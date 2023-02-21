use pest::iterators::Pair;

use super::{Expression, Ident, Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Definition<T> {
    pub ident: Ident<T>,
    pub value: Expression<T>,
    pub position: Position,
    pub is_mutable: bool,
    pub info: T,
}

impl Definition<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Definition<()> {
        let mut inner = pair.clone().into_inner();

        let (line, col) = pair.line_col();

        let mut is_mutable = false;

        let ident_or_mut = inner.next().unwrap_or_else(|| {
            panic!(
                "Expected lvalue or 'mut' in definition '{}' at {}:{}",
                pair.as_str(),
                pair.line_col().0,
                pair.line_col().1
            )
        });

        let ident = if ident_or_mut.as_rule() == Rule::mutKeyword {
            is_mutable = true;
            inner.next().unwrap_or_else(|| {
                panic!(
                    "Expected lvalue in definition '{}' at {}:{}",
                    pair.as_str(),
                    pair.line_col().0,
                    pair.line_col().1
                )
            })
        } else {
            ident_or_mut
        };

        let ident = Ident::from_pair(ident, file);

        let value = inner.next().unwrap_or_else(|| {
            panic!(
                "Expected rvalue in definition '{}' at {}:{}",
                pair.as_str(),
                pair.line_col().0,
                pair.line_col().1
            )
        });
        let value = Expression::from_pair(value, file);

        Definition {
            ident,
            value,
            position: (file.to_owned(), line, col),
            is_mutable,
            info: (),
        }
    }
}
