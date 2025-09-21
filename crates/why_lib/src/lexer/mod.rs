use colored::Colorize;

#[derive(Default, Debug, Clone, Eq, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub source: String,
}

pub fn indices_to_row_col(s: &str, start: usize, end: usize) -> ((usize, usize), (usize, usize)) {
    let max_index = start.max(end);
    let mut line_starts = vec![0]; // First line starts at byte 0

    // Collect line start positions up to max_index
    if max_index > 0 {
        for (i, c) in s.char_indices() {
            if i > max_index {
                break; // No need to process beyond max_index
            }
            if c == '\n' {
                line_starts.push(i + 1); // Next line starts after '\n'
            }
        }
    }

    // Helper to convert a byte index to (row, col)
    let get_pos = |pos: usize| {
        // Find the last line start <= pos
        let row_index = line_starts.partition_point(|&x| x <= pos) - 1;
        let col = pos - line_starts[row_index];
        (row_index, col)
    };

    (get_pos(start), get_pos(end))
}

impl Span {
    pub fn new((start, end): (usize, usize), source: &str) -> Self {
        let (start, end) = indices_to_row_col(source, start, end);

        Self {
            start,
            end,
            source: source.to_string(),
        }
    }

    pub fn to_string(&self, msg: impl ToString) -> String {
        let Span { start, end, source } = self;
        let line = start.0;
        let lines = source.lines().collect::<Vec<_>>();
        let prev_line = if line > 0 { lines[line - 1] } else { "" };
        let line_str = if lines.len() > line { lines[line] } else { "" };

        // margin _before_ left border
        let left_margin = format!("{}", end.0).len();
        let left_margin_fill = vec![' '; left_margin].iter().collect::<String>();

        // split right at the start of the error in the first line
        let (left, right) = line_str.split_at(start.1);

        // some case magic
        let (left, right) = if start.0 != end.0 {
            // if the error ranges over more than a single line, we can just mark rest of the line
            // as an error
            (left.to_string(), right.to_string().red().to_string())
        } else {
            // however, if the lines does not range beyond this line, we need to split at the end
            // again
            let (err_str, after_err) = right.split_at(end.1 - start.1);

            // now, just color the error part red
            (
                left.to_string(),
                format!("{err_str}{after_err}", err_str = err_str.to_string().red()),
            )
        };

        // and concatentate both together
        let line_str = format!("{left}{right}");

        // padding between border and squiggles
        let left_padding_fill = vec![' '; if end.1 > 0 { end.1 - 1 } else { 0 }]
            .iter()
            .collect::<String>();

        // the error with the first line
        let mut error_string = format!(
            "{left_margin_fill} |\n{left_margin_fill} |{prev_line} \n{line} |{line_str}",
            line = line + 1
        );

        // iterate over all lines of the error and make them shine red
        ((start.0 + 1)..(end.0 + 1)).for_each(|line_number| {
            error_string = format!(
                "{error_string}\n{left_margin_fill} |{}",
                lines[line_number].to_string().red()
            );
        });

        // actually add error message at bottom
        error_string = format!(
            "{error_string}\n{} |{left_padding_fill}^--- {}\n{left_margin_fill} |",
            end.0 + 2,
            msg.to_string()
        );

        error_string
    }

    pub fn merge(&self, other: &Span) -> Span {
        let Span { start, source, .. } = self.clone();
        let Span { end, .. } = other.clone();

        Span { start, end, source }
    }
}

impl PartialEq<Span> for Span {
    fn eq(&self, _other: &Span) -> bool {
        // TODO: maybe this should not be the case...
        true
    }
}
