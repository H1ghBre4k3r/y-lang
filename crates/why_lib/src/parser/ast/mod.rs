mod expression;
mod statement;
mod type_name;

pub use self::expression::*;
pub use self::statement::*;
pub use self::type_name::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AstNode {
    Expression(Expression<()>),
    Id(Id<()>),
    Num(Num<()>),
    Bool(Bool<()>),
    Character(Character<()>),
    AstString(AstString<()>),
    Statement(Statement<()>),
    Initialization(Initialisation<()>),
    Constant(Constant<()>),
    Assignment(Assignment<()>),
    Function(Function<()>),
    Lambda(Lambda<()>),
    LambdaParameter(LambdaParameter<()>),
    If(If<()>),
    WhileLoop(WhileLoop<()>),
    FunctionParameter(FunctionParameter<()>),
    TypeName(TypeName),
    Block(Block<()>),
    Array(Array<()>),
    Declaration(Declaration<()>),
    StructDeclaration(StructDeclaration<()>),
    StructFieldDeclaration(StructFieldDeclaration<()>),
    StructInitialisation(StructInitialisation<()>),
    StructFieldInitialisation(StructFieldInitialisation<()>),
    Instance(Instance<()>),
    MethodDeclaration(MethodDeclaration<()>),
}
