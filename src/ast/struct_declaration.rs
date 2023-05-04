use super::{Ident, Position, Rule, TypeAnnotation};
use pest::iterators::Pair;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructDeclaration<T> {
    pub ident: Ident<T>,
    pub members: Vec<StructMember<T>>,
    pub info: T,
    pub position: Position,
}

impl StructDeclaration<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> StructDeclaration<()> {
        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();

        let ident = Ident::from_pair(
            inner
                .next()
                .unwrap_or_else(|| panic!("Expected identifier in struct declaration")),
            file,
        );

        let mut members = vec![];

        for struct_member in inner {
            members.push(StructMember::from_pair(struct_member, file));
        }

        StructDeclaration {
            ident,
            info: (),
            members,
            position: (file.to_owned(), line, col),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructMember<T> {
    pub ident: Ident<T>,
    pub _type: TypeAnnotation,
    pub info: T,
    pub position: Position,
}

impl StructMember<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> StructMember<()> {
        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();

        let ident = Ident::from_pair(
            inner
                .next()
                .unwrap_or_else(|| panic!("Expected identifier in struct member")),
            file,
        );

        let type_annotation = TypeAnnotation::from_pair(
            inner
                .next()
                .unwrap_or_else(|| panic!("Expected type annotation in struct member")),
            file,
        );

        StructMember {
            ident,
            _type: type_annotation,
            info: (),
            position: (file.to_owned(), line, col),
        }
    }
}
