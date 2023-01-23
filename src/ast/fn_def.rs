use log::error;
use pest::iterators::Pair;

use super::{Block, Param, Position, Rule, TypeAnnotation};

#[derive(Debug, Clone)]
pub struct FnDef {
    pub params: Vec<Param>,
    pub type_annotation: TypeAnnotation,
    pub block: Block,
    pub position: Position,
}

impl FnDef {
    pub fn from_pair(pair: Pair<Rule>) -> FnDef {
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
        let type_annotation = TypeAnnotation::from_pair(type_annotation);

        let Some(block) = inner.next() else {
            error!("Expected block in function definition at {}:{}", position.0, position.1);
            std::process::exit(-1);
        };
        let block = Block::from_pair(block);

        FnDef {
            params: param_list,
            type_annotation,
            block,
            position,
        }
    }

    fn from_param_list(pair: Pair<Rule>) -> Vec<Param> {
        assert_eq!(pair.as_rule(), Rule::paramList);

        let param_pairs = pair.into_inner();

        let mut params = vec![];

        for param in param_pairs {
            params.push(Param::from_pair(param));
        }

        params
    }
}
