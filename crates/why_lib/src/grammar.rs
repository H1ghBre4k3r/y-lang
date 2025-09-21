#[rust_sitter::grammar("ylang_grammar")]
#[allow(dead_code)]
#[allow(clippy::manual_non_exhaustive)]
mod ylang_grammar {
    use rust_sitter::Spanned;

    #[rust_sitter::language]
    #[derive(Debug, Clone)]
    pub struct Program {
        #[rust_sitter::repeat]
        pub statements: Vec<Spanned<ToplevelStatement>>,
    }

    #[derive(Debug, Clone)]
    pub enum ToplevelStatement {
        FunctionDeclaration(Spanned<FunctionDeklaration>),
        Constant(Spanned<Constant>),
        Declaration(Spanned<Declaration>),
        StructDeclaration(Spanned<StructDeclaration>),
        Instance(Spanned<Instance>),
        Comment(Spanned<Comment>),
    }

    #[derive(Debug, Clone)]
    pub enum Statement {
        FunctionDeclaration(Spanned<FunctionDeklaration>),
        VariableDeclaration(Spanned<VariableDeclaration>),
        Assignment(Spanned<Assignment>),
        WhileStatement(Spanned<WhileStatement>),
        Constant(Spanned<Constant>),
        Expression {
            inner: Spanned<Expression>,
            #[rust_sitter::leaf(text = ";")]
            _semicolon: (),
        },
        #[rust_sitter::prec_left(5)]
        YieldingExpression(Spanned<Expression>),
        Return {
            #[rust_sitter::leaf(text = "return")]
            _return: (),
            inner: Spanned<Expression>,
            #[rust_sitter::leaf(text = ";")]
            _semicolon: (),
        },
        Declaration(Spanned<Declaration>),
        StructDeclaration(Spanned<StructDeclaration>),
        Comment(Spanned<Comment>),
    }

    #[derive(Debug, Clone)]
    pub enum Expression {
        Boolean(Spanned<BooleanLiteral>),
        Identifier(Spanned<Identifier>),
        Number(Spanned<Number>),
        String(Spanned<StringLiteral>),
        Character(Spanned<CharacterLiteral>),
        IfExpression(Spanned<IfExpression>),
        Parenthesized(Spanned<ParenthesizedExpression>),
        BinaryExpression(Spanned<BinaryExpression>),
        Block(Spanned<Block>),
        Lambda(Spanned<Lambda>),
        Postfix(Spanned<Postfix>),
        Prefix(Spanned<Prefix>),
        Array(Spanned<Array>),
        StructInitialisation(Spanned<StructInitialisation>),
    }

    #[derive(Debug, Clone)]
    #[rust_sitter::word]
    pub struct Identifier(
        #[rust_sitter::leaf(pattern = "[_a-zA-Z][_a-zA-Z0-9]*", transform = |v| v.to_string())]
        pub Spanned<String>,
    );

    #[derive(Debug, Clone)]
    pub enum Number {
        Integer(Integer),
        Floating(Floating),
    }

    #[derive(Debug, Clone)]
    pub struct Integer(
        #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] pub u64,
    );

    #[derive(Debug, Clone)]
    pub struct Floating(
        #[rust_sitter::leaf(pattern = r"\d+\.\d+", transform = |v| v.parse().unwrap())] pub f64,
    );

    #[derive(Debug, Clone)]
    pub struct StringLiteral(
        #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#, transform = |v| {
            let trimmed = v.strip_prefix('"').unwrap().strip_suffix('"').unwrap();
            trimmed.to_string()
        })]
        pub Spanned<String>,
    );

    #[derive(Debug, Clone)]
    pub struct CharacterLiteral(
        #[rust_sitter::leaf(pattern = r"'([^'\\]|\\.)'", transform = |v| {
            let trimmed = v.strip_prefix('\'').unwrap().strip_suffix('\'').unwrap();
            trimmed.chars().next().unwrap()
        })]
        pub Spanned<char>,
    );

    #[derive(Debug, Clone)]
    pub enum BooleanLiteral {
        #[rust_sitter::leaf(text = "true")]
        True,
        #[rust_sitter::leaf(text = "false")]
        False,
    }

    #[derive(Debug, Clone)]
    pub struct ParenthesizedExpression {
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub inner: Box<Spanned<Expression>>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
    }

    #[derive(Debug, Clone)]
    pub enum BinaryExpression {
        #[rust_sitter::prec_left(1)]
        Addition(
            Box<Spanned<Expression>>,
            #[rust_sitter::leaf(text = "+")] (),
            Box<Spanned<Expression>>,
        ),
        #[rust_sitter::prec_left(1)]
        Subtraction(
            Box<Spanned<Expression>>,
            #[rust_sitter::leaf(text = "-")] (),
            Box<Spanned<Expression>>,
        ),
        #[rust_sitter::prec_left(2)]
        Multiplication(
            Box<Spanned<Expression>>,
            #[rust_sitter::leaf(text = "*")] (),
            Box<Spanned<Expression>>,
        ),
        #[rust_sitter::prec_left(2)]
        Division(
            Box<Spanned<Expression>>,
            #[rust_sitter::leaf(text = "/")] (),
            Box<Spanned<Expression>>,
        ),
        #[rust_sitter::prec_left(0)]
        Equals(
            Box<Spanned<Expression>>,
            #[rust_sitter::leaf(text = "==")] (),
            Box<Spanned<Expression>>,
        ),
        #[rust_sitter::prec_left(0)]
        NotEquals(
            Box<Spanned<Expression>>,
            #[rust_sitter::leaf(text = "!=")] (),
            Box<Spanned<Expression>>,
        ),
        #[rust_sitter::prec_left(0)]
        LessThan(
            Box<Spanned<Expression>>,
            #[rust_sitter::leaf(text = "<")] (),
            Box<Spanned<Expression>>,
        ),
        #[rust_sitter::prec_left(0)]
        GreaterThan(
            Box<Spanned<Expression>>,
            #[rust_sitter::leaf(text = ">")] (),
            Box<Spanned<Expression>>,
        ),
        #[rust_sitter::prec_left(0)]
        LessOrEqual(
            Box<Spanned<Expression>>,
            #[rust_sitter::leaf(text = "<=")] (),
            Box<Spanned<Expression>>,
        ),
        #[rust_sitter::prec_left(0)]
        GreaterOrEqual(
            Box<Spanned<Expression>>,
            #[rust_sitter::leaf(text = ">=")] (),
            Box<Spanned<Expression>>,
        ),
    }

    #[derive(Debug, Clone)]
    pub struct VariableDeclaration {
        #[rust_sitter::leaf(text = "let")]
        _let: (),
        #[rust_sitter::optional]
        pub mutability: Option<MutabilityKeyword>,
        pub identifier: Spanned<Identifier>,
        #[rust_sitter::optional]
        pub type_annotation: Option<TypeAnnotation>,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Spanned<Expression>,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[derive(Debug, Clone)]
    pub struct MutabilityKeyword {
        #[rust_sitter::leaf(text = "mut")]
        _mut: (),
    }

    #[derive(Debug, Clone)]
    pub struct Assignment {
        pub lvalue: Spanned<LValue>,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Spanned<Expression>,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[derive(Debug, Clone)]
    pub enum LValue {
        Identifier(Spanned<Identifier>),
        PropertyAccess(Spanned<PropertyAccess>),
        IndexExpression(Spanned<IndexExpression>),
    }

    #[derive(Debug, Clone)]
    pub struct Declaration {
        #[rust_sitter::leaf(text = "declare")]
        _declare: (),
        pub name: Spanned<Identifier>,
        pub type_annotation: TypeAnnotation,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[derive(Debug, Clone)]
    pub struct StructDeclaration {
        #[rust_sitter::leaf(text = "struct")]
        _struct: (),
        pub id: Spanned<Identifier>,
        #[rust_sitter::leaf(text = "{")]
        _lbrace: (),
        #[rust_sitter::repeat]
        pub fields: Vec<Spanned<StructFieldDeclaration>>,
        #[rust_sitter::leaf(text = "}")]
        _rbrace: (),
    }

    #[derive(Debug, Clone)]
    pub struct StructFieldDeclaration {
        pub name: Spanned<Identifier>,
        pub type_annotation: TypeAnnotation,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[derive(Debug, Clone)]
    pub struct Instance {
        #[rust_sitter::leaf(text = "instance")]
        _instance: (),
        pub name: Spanned<TypeName>,
        #[rust_sitter::leaf(text = "{")]
        _lbrace: (),
        #[rust_sitter::repeat]
        pub methods: Vec<InstanceMethod>,
        #[rust_sitter::leaf(text = "}")]
        _rbrace: (),
    }

    #[derive(Debug, Clone)]
    pub enum InstanceMethod {
        FunctionDeclaration(Spanned<FunctionDeklaration>),
        MethodDeclaration(Spanned<MethodDeclaration>),
    }

    #[derive(Debug, Clone)]
    pub struct MethodDeclaration {
        #[rust_sitter::leaf(text = "declare")]
        _declare: (),
        pub id: Spanned<Identifier>,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub parameter_types: Vec<Spanned<TypeName>>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub return_type_annotation: TypeAnnotation,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[derive(Debug, Clone)]
    pub struct IfExpression {
        #[rust_sitter::leaf(text = "if")]
        _if: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub condition: Box<Spanned<Expression>>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub then_block: Spanned<Spanned<Block>>,
        #[rust_sitter::optional]
        pub else_block: Option<Spanned<ElseClause>>,
    }

    #[derive(Debug, Clone)]
    pub struct ElseClause {
        #[rust_sitter::leaf(text = "else")]
        _else: (),
        pub block: Spanned<Spanned<Block>>,
    }

    #[derive(Debug, Clone)]
    pub struct WhileStatement {
        #[rust_sitter::leaf(text = "while")]
        _while: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub condition: Box<Spanned<Expression>>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub block: Spanned<Spanned<Block>>,
    }

    #[derive(Debug, Clone)]
    pub struct FunctionDeklaration {
        #[rust_sitter::leaf(text = "fn")]
        _keyword: (),
        pub ident: Spanned<Identifier>,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub parameters: Vec<Spanned<FunctionParameter>>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub type_annotation: TypeAnnotation,
        pub block: Spanned<Block>,
    }

    #[derive(Debug, Clone)]
    pub struct FunctionParameter {
        pub ident: Spanned<Identifier>,
        pub type_annotation: TypeAnnotation,
    }

    #[derive(Debug, Clone)]
    pub struct TypeAnnotation {
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub type_name: Spanned<TypeName>,
    }

    #[derive(Debug, Clone)]
    pub enum TypeName {
        LiteralType(LiteralType),
        ArrayType(ArrayType),
        ReferenceType(ReferenceType),
        FunctionType(FunctionType),
        TupleType(TupleType),
    }

    #[derive(Debug, Clone)]
    pub struct LiteralType {
        pub typename: Spanned<Identifier>,
    }

    #[derive(Debug, Clone)]
    pub struct ArrayType {
        #[rust_sitter::leaf(text = "&[")]
        _lbracket: (),
        pub inner: Box<Spanned<TypeName>>,
        #[rust_sitter::leaf(text = "]")]
        _rbracket: (),
    }

    #[derive(Debug, Clone)]
    pub struct ReferenceType {
        #[rust_sitter::leaf(text = "&")]
        _ampersand: (),
        pub inner: Box<Spanned<TypeName>>,
    }

    #[derive(Debug, Clone)]
    pub struct TupleType {
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
        pub types: Vec<Spanned<TypeName>>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
    }

    #[derive(Debug, Clone)]
    pub struct FunctionType {
        pub params: TupleType,
        #[rust_sitter::leaf(text = "->")]
        _arrow: (),
        pub return_type: Box<Spanned<TypeName>>,
    }

    #[derive(Debug, Clone)]
    pub struct Block {
        #[rust_sitter::leaf(text = "{")]
        _lbrace: (),
        pub statements: Vec<Spanned<Statement>>,
        #[rust_sitter::leaf(text = "}")]
        _rbrace: (),
    }

    #[derive(Debug, Clone)]
    pub struct Constant {
        #[rust_sitter::leaf(text = "const")]
        _const: (),
        pub identifier: Spanned<Identifier>,
        pub type_annotation: TypeAnnotation,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Spanned<Expression>,
        #[rust_sitter::leaf(text = ";")]
        _semi: (),
    }

    #[derive(Debug, Clone)]
    #[rust_sitter::prec_right(0)]
    pub struct Lambda {
        #[rust_sitter::leaf(text = "\\")]
        _start: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
        pub params: Vec<Spanned<LambdaParameter>>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        #[rust_sitter::leaf(text = "=>")]
        _rarrow: (),
        pub expression: Box<Spanned<Expression>>,
    }

    #[derive(Debug, Clone)]
    pub struct LambdaParameter {
        pub ident: Spanned<Identifier>,
        #[rust_sitter::optional]
        pub type_annotation: Option<TypeAnnotation>,
    }

    #[derive(Debug, Clone)]
    #[rust_sitter::prec_left(10)]
    pub enum Postfix {
        Call(CallExpression),
        Index(IndexExpression),
        PropertyAccess(PropertyAccess),
    }

    #[derive(Debug, Clone)]
    #[rust_sitter::prec_left(10)]
    pub struct CallExpression {
        pub expression: Box<Spanned<Expression>>,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
        pub args: Vec<Spanned<Expression>>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
    }

    #[derive(Debug, Clone)]
    #[rust_sitter::prec_left(10)]
    pub struct IndexExpression {
        pub expression: Box<Spanned<Expression>>,
        #[rust_sitter::leaf(text = "[")]
        _lbracket: (),
        pub index: Box<Spanned<Expression>>,
        #[rust_sitter::leaf(text = "]")]
        _rbracket: (),
    }

    #[derive(Debug, Clone)]
    #[rust_sitter::prec_left(10)]
    pub struct PropertyAccess {
        pub expression: Box<Spanned<Expression>>,
        #[rust_sitter::leaf(text = ".")]
        _dot: (),
        pub property: Spanned<Identifier>,
    }

    #[derive(Debug, Clone)]
    pub enum Prefix {
        #[rust_sitter::prec_right(3)]
        Negation {
            #[rust_sitter::leaf(text = "!")]
            _not: (),
            expression: Box<Spanned<Expression>>,
        },
        #[rust_sitter::prec_right(3)]
        Minus {
            #[rust_sitter::leaf(text = "-")]
            _minus: (),
            expression: Box<Spanned<Expression>>,
        },
    }

    #[derive(Debug, Clone)]
    #[rust_sitter::prec_right(3)]
    pub struct Array {
        #[rust_sitter::leaf(text = "&[")]
        _lbracket: (),
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub elements: Vec<Spanned<Expression>>,
        #[rust_sitter::leaf(text = "]")]
        _rbracket: (),
    }

    #[derive(Debug, Clone)]
    #[rust_sitter::prec_left(4)]
    pub struct StructInitialisation {
        pub id: Spanned<Identifier>,
        #[rust_sitter::leaf(text = "{")]
        _lbrace: (),
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub fields: Vec<Spanned<StructFieldInitialisation>>,
        #[rust_sitter::leaf(text = "}")]
        _rbrace: (),
    }

    #[derive(Debug, Clone)]
    pub struct StructFieldInitialisation {
        pub name: Spanned<Identifier>,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub value: Spanned<Expression>,
    }

    #[derive(Debug, Clone)]
    pub struct Comment {
        #[rust_sitter::leaf(pattern = r"//[^\r\n]*", transform = |v| v.to_string())]
        pub content: String,
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

pub use self::ylang_grammar::*;

pub trait FromGrammar<T> {
    fn transform(item: rust_sitter::Spanned<T>, source: &str) -> Self;
}
