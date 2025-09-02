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
        // Constant,
        // Declaration,
        // StructDeclaration,
        // Instance,
    }

    #[derive(Debug)]
    pub enum Statement {
        FunctionDeclaration(Spanned<FunctionDeklaration>),
        // IfStatement,
        // WhileLoop,
        // Initialization,
        // Constant,
        // Assignment,
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
        #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] Spanned<f64>,
    );

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
        Literal {
            typename: Identifier,
        },
        ArrayType {
            #[rust_sitter::leaf(text = "&")]
            _ref: (),
            #[rust_sitter::leaf(text = "[")]
            _lbracket: (),
            inner: Identifier,
            #[rust_sitter::leaf(text = "]")]
            _rbracket: (),
        },
        FunctionType {
            #[rust_sitter::leaf(text = "fn")]
            _fn: (),
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            params: Vec<TypeName>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
            #[rust_sitter::leaf(text = "->")]
            _arrow: (),
            return_type: Box<TypeName>,
        },
        TupleType {
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            types: Vec<TypeName>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
        },
    }

    #[derive(Debug)]
    pub struct Block {
        #[rust_sitter::leaf(text = "{")]
        _lbrace: (),
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "}")]
        _rbrace: (),
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

pub use self::ylang_grammar::*;
