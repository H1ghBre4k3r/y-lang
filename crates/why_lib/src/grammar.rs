#[rust_sitter::grammar("ylang_grammar")]
#[allow(dead_code)]
mod ylang_grammar {
    use rust_sitter::Spanned;

    #[rust_sitter::language]
    #[derive(Debug)]
    pub struct Program {
        #[rust_sitter::repeat]
        pub statements: Vec<ToplevelStatement>,
    }

    #[derive(Debug)]
    pub enum ToplevelStatement {
        FunctionDeclaration(Spanned<FunctionDeklaration>),
        Constant(Constant),
        // Declaration,
        // StructDeclaration,
        // Instance,
    }

    #[derive(Debug)]
    pub enum Statement {
        FunctionDeclaration(Spanned<FunctionDeklaration>),
        VariableDeclaration(VariableDeclaration),
        Assignment(Assignment),
        While(WhileStatement),
        Constant(Constant),
        Expression {
            inner: Expression,
            #[rust_sitter::leaf(text = ";")]
            _semicolon: (),
        },
        YieldingExpression(Expression),
        Return {
            #[rust_sitter::leaf(text = "return")]
            _return: (),
            inner: Expression,
            #[rust_sitter::leaf(text = ";")]
            _semicolon: (),
        },
        // Comment,
        // Declaration,
        // StructDeclaration,
    }

    #[derive(Debug)]
    pub enum Expression {
        Identifier(Identifier),
        Number(Number),
        String(StringLiteral),
        Character(CharacterLiteral),
        IfExpression(IfExpression),
        Parenthesized(ParenthesizedExpression),
        BinaryExpression(BinaryExpression),
        Block(Block),
        // Lambda,
        // Postfix,
        // Prefix,
        // Array,
        // StructInitialisation
    }

    #[derive(Debug)]
    pub struct Identifier(
        #[rust_sitter::leaf(pattern = r"[_a-zA-z][_a-zA-Z0-9]*", transform = |v| v.to_string())]
        Spanned<String>,
    );

    #[derive(Debug)]
    pub enum Number {
        Integer(Integer),
        Floating(Floating),
    }

    #[derive(Debug)]
    pub struct Integer(
        #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] Spanned<i64>,
    );

    #[derive(Debug)]
    pub struct Floating(
        #[rust_sitter::leaf(pattern = r"\d+\.\d+", transform = |v| v.parse().unwrap())]
        Spanned<f64>,
    );

    #[derive(Debug)]
    pub struct StringLiteral(
        #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#, transform = |v| {
            let trimmed = v.strip_prefix('"').unwrap().strip_suffix('"').unwrap();
            trimmed.to_string()
        })]
        Spanned<String>,
    );

    #[derive(Debug)]
    pub struct CharacterLiteral(
        #[rust_sitter::leaf(pattern = r"'([^'\\]|\\.)'", transform = |v| {
            let trimmed = v.strip_prefix('\'').unwrap().strip_suffix('\'').unwrap();
            trimmed.chars().next().unwrap()
        })]
        Spanned<char>,
    );

    #[derive(Debug)]
    pub struct ParenthesizedExpression {
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub inner: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
    }

    #[derive(Debug)]
    pub enum BinaryExpression {
        #[rust_sitter::prec_left(1)]
        Addition(
            Box<Expression>,
            #[rust_sitter::leaf(text = "+")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(1)]
        Subtraction(
            Box<Expression>,
            #[rust_sitter::leaf(text = "-")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(2)]
        Multiplication(
            Box<Expression>,
            #[rust_sitter::leaf(text = "*")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(2)]
        Division(
            Box<Expression>,
            #[rust_sitter::leaf(text = "/")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        Equals(
            Box<Expression>,
            #[rust_sitter::leaf(text = "==")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        NotEquals(
            Box<Expression>,
            #[rust_sitter::leaf(text = "!=")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        LessThan(
            Box<Expression>,
            #[rust_sitter::leaf(text = "<")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        GreaterThan(
            Box<Expression>,
            #[rust_sitter::leaf(text = ">")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        LessOrEqual(
            Box<Expression>,
            #[rust_sitter::leaf(text = "<=")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        GreaterOrEqual(
            Box<Expression>,
            #[rust_sitter::leaf(text = ">=")] (),
            Box<Expression>,
        ),
    }

    #[derive(Debug)]
    pub struct VariableDeclaration {
        #[rust_sitter::leaf(text = "let")]
        _let: (),
        #[rust_sitter::optional]
        pub mutability: Option<MutabilityKeyword>,
        pub identifier: Identifier,
        #[rust_sitter::optional]
        pub type_annotation: Option<TypeAnnotation>,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Expression,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[derive(Debug)]
    pub struct MutabilityKeyword {
        #[rust_sitter::leaf(text = "mut")]
        _mut: (),
    }

    #[derive(Debug)]
    pub struct Assignment {
        pub identifier: Identifier,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Expression,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[derive(Debug)]
    pub struct IfExpression {
        #[rust_sitter::leaf(text = "if")]
        _if: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub condition: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub then_block: Spanned<Block>,
        #[rust_sitter::optional]
        pub else_block: Option<ElseClause>,
    }

    #[derive(Debug)]
    pub struct ElseClause {
        #[rust_sitter::leaf(text = "else")]
        _else: (),
        pub block: Spanned<Block>,
    }

    #[derive(Debug)]
    pub struct WhileStatement {
        #[rust_sitter::leaf(text = "while")]
        _while: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub condition: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub block: Spanned<Block>,
    }

    #[derive(Debug)]
    pub struct FunctionDeklaration {
        #[rust_sitter::leaf(text = "fn")]
        _keyword: (),
        pub ident: Identifier,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub parameters: Vec<FunctionParameter>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub type_annotation: TypeAnnotation,
        pub block: Spanned<Block>,
    }

    #[derive(Debug)]
    pub struct FunctionParameter {
        pub ident: Identifier,
        pub type_annotation: TypeAnnotation,
    }

    #[derive(Debug)]
    pub struct TypeAnnotation {
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub type_name: Spanned<TypeName>,
    }

    #[derive(Debug)]
    pub enum TypeName {
        LiteralType(LiteralType),
        ArrayType(ArrayType),
        FunctionType(FunctionType),
        TupleType(TupleType),
    }

    #[derive(Debug)]
    pub struct LiteralType {
        typename: Identifier,
    }

    #[derive(Debug)]
    pub struct ArrayType {
        #[rust_sitter::leaf(text = "&")]
        _ref: (),
        #[rust_sitter::leaf(text = "[")]
        _lbracket: (),
        pub inner: Identifier,
        #[rust_sitter::leaf(text = "]")]
        _rbracket: (),
    }

    #[derive(Debug)]
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

    #[derive(Debug)]
    pub struct FunctionType {
        pub params: TupleType,
        #[rust_sitter::leaf(text = "->")]
        _arrow: (),
        pub return_type: Box<Spanned<TypeName>>,
    }

    #[derive(Debug)]
    pub struct Block {
        #[rust_sitter::leaf(text = "{")]
        _lbrace: (),
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "}")]
        _rbrace: (),
    }

    #[derive(Debug)]
    pub struct Constant {
        #[rust_sitter::leaf(text = "const")]
        _const: (),
        pub identifier: Identifier,
        pub type_annotation: TypeAnnotation,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Expression,
        #[rust_sitter::leaf(text = ";")]
        _semi: (),
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

pub use self::ylang_grammar::*;
