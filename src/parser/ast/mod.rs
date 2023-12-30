mod expression;
mod statement;
mod type_name;

pub use self::expression::*;
pub use self::statement::*;
pub use self::type_name::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNode {
    Expression(Expression),
    Id(Id),
    Num(Num),
    Statement(Statement),
    Initialization(Initialisation),
    Constant(Constant),
    Assignment(Assignment),
    Function(Function),
    Lambda(Lambda),
    If(If),
    WhileLoop(WhileLoop),
    Parameter(Parameter),
    TypeName(TypeName),
    Block(Block),
    Array(Array),
    Declaration(Declaration),
    StructDeclaration(StructDeclaration),
    StructFieldDeclaration(StructFieldDeclaration),
    StructInitialisation(StructInitialisation),
    StructFieldInitialisation(StructFieldInitialisation),
}
