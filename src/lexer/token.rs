use pesca_parser_derive::Token as ParseToken;

type Position = (usize, usize);

#[derive(Debug, Clone, ParseToken)]
pub enum Token {
    #[terminal]
    Eq {
        position: Position,
    },
    #[terminal]
    Let {
        position: Position,
    },
    Id {
        value: String,
        position: Position,
    },
    Num {
        value: u64,
        position: Position,
    },
    #[terminal]
    Semicolon {
        position: Position,
    },
    // TODO: think about lexing comments
    Comment {
        value: String,
        position: Position,
    },
    #[terminal]
    Plus {
        position: Position,
    },
    #[terminal]
    Times {
        position: Position,
    },
    #[terminal]
    LParen {
        position: Position,
    },
    #[terminal]
    RParen {
        position: Position,
    },
    #[terminal]
    LBrace {
        position: Position,
    },
    #[terminal]
    RBrace {
        position: Position,
    },
    #[terminal]
    FnKeyword {
        position: Position,
    },
    #[terminal]
    IfKeyword {
        position: Position,
    },
    #[terminal]
    ElseKeyword {
        position: Position,
    },
    #[terminal]
    ReturnKeyword {
        position: Position,
    },
    #[terminal]
    Colon {
        position: Position,
    },
    #[terminal]
    Comma {
        position: Position,
    },
}

impl Terminal {
    pub fn to_token(&self, position: Position) -> Token {
        match self {
            Terminal::Eq => Token::Eq { position },
            Terminal::Let => Token::Let { position },
            Terminal::Semicolon => Token::Semicolon { position },
            Terminal::Plus => Token::Plus { position },
            Terminal::Times => Token::Times { position },
            Terminal::LParen => Token::LParen { position },
            Terminal::RParen => Token::RParen { position },
            Terminal::LBrace => Token::LBrace { position },
            Terminal::RBrace => Token::RBrace { position },
            Terminal::FnKeyword => Token::FnKeyword { position },
            Terminal::IfKeyword => Token::IfKeyword { position },
            Terminal::ElseKeyword => Token::ElseKeyword { position },
            Terminal::ReturnKeyword => Token::ReturnKeyword { position },
            Terminal::Colon => Token::Colon { position },
            Terminal::Comma => Token::Comma { position },
        }
    }
}

// TODO: move this to own derive macro
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        use Token::*;
        matches!(
            (self, other),
            (Eq { .. }, Eq { .. })
                | (Let { .. }, Let { .. })
                | (Id { .. }, Id { .. })
                | (Num { .. }, Num { .. })
                | (Semicolon { .. }, Semicolon { .. })
                | (Comment { .. }, Comment { .. })
                | (Plus { .. }, Plus { .. })
                | (Times { .. }, Times { .. })
                | (LParen { .. }, LParen { .. })
                | (RParen { .. }, RParen { .. })
                | (LBrace { .. }, LBrace { .. })
                | (RBrace { .. }, RBrace { .. })
                | (FnKeyword { .. }, FnKeyword { .. })
                | (IfKeyword { .. }, IfKeyword { .. })
                | (ElseKeyword { .. }, ElseKeyword { .. })
                | (ReturnKeyword { .. }, ReturnKeyword { .. })
                | (Colon { .. }, Colon { .. })
                | (Comma { .. }, Comma { .. })
        )
    }
}

impl Eq for Token {}

impl Token {
    pub fn position(&self) -> Position {
        match self {
            Token::Eq { position } => *position,
            Token::Let { position } => *position,
            Token::Id { position, .. } => *position,
            Token::Num { position, .. } => *position,
            Token::Semicolon { position } => *position,
            Token::Comment { position, .. } => *position,
            Token::Plus { position } => *position,
            Token::Times { position } => *position,
            Token::LParen { position } => *position,
            Token::RParen { position } => *position,
            Token::LBrace { position } => *position,
            Token::RBrace { position } => *position,
            Token::FnKeyword { position } => *position,
            Token::IfKeyword { position } => *position,
            Token::ElseKeyword { position } => *position,
            Token::ReturnKeyword { position } => *position,
            Token::Colon { position } => *position,
            Token::Comma { position } => *position,
        }
    }
}
