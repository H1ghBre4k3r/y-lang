use log::{error, trace};
use pest::iterators::Pair;

use super::{Block, Param, Position, Rule, TypeAnnotation};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FnDef<T> {
    pub params: Vec<Param<T>>,
    pub type_annotation: TypeAnnotation,
    pub block: Block<T>,
    pub position: Position,
    pub info: T,
}

impl FnDef<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> FnDef<()> {
        assert_eq!(pair.as_rule(), Rule::fnDef);
        trace!("creating FnDef form pair '{pair:?}'");

        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();

        let Some(param_list) = inner.next() else {
            error!("Expected param list in function definition at {}:{}", line, col);
            std::process::exit(-1);
        };
        let param_list = Self::from_param_list(param_list, file);

        let Some(type_annotation) = inner.next() else {
            error!("Expected return type annotation in function definition at {}:{}", line, col);
            std::process::exit(-1);
        };
        let type_annotation = TypeAnnotation::from_pair(type_annotation, file);

        let Some(block) = inner.next() else {
            error!("Expected block in function definition at {}:{}", line, col);
            std::process::exit(-1);
        };
        let block = Block::from_pair(block, file);

        FnDef {
            params: param_list,
            type_annotation,
            block,
            position: (file.to_owned(), line, col),
            info: (),
        }
    }

    fn from_param_list(pair: Pair<Rule>, file: &str) -> Vec<Param<()>> {
        assert_eq!(pair.as_rule(), Rule::paramList);

        let param_pairs = pair.into_inner();

        let mut params = vec![];

        for param in param_pairs {
            params.push(Param::from_pair(param, file));
        }

        params
    }
}
