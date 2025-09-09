use rust_sitter::errors::ParseErrorReason;
use why_lib::lexer::Span;

pub fn convert_parse_error(
    error: rust_sitter::errors::ParseError,
    input: &str,
    analysis: &mut Vec<(String, Span)>,
) {
    let span = Span::new((error.start, error.end), input);

    match error.reason {
        ParseErrorReason::UnexpectedToken(msg) => {
            analysis.push((format!("Unexpected token: {msg}"), span));
        }
        ParseErrorReason::MissingToken(msg) => {
            analysis.push((format!("expected {msg}"), span));
        }
        ParseErrorReason::FailedNode(errors) => {
            analysis.push(("Failed to parse!".into(), span));
            for e in errors {
                convert_parse_error(e, input, analysis);
            }
        }
    }
}
