mod lexer;

use self::lexer::lex;

fn main() {
    let input = r#"
        // some comment
        let a = 3;
        "#;

    println!("{:#?}", lex(input));
}
