use crate::parser::ParseError;

/// Struct for iterating over a vector of tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseState<T> {
    tokens: Vec<T>,
    index: usize,
    pub errors: Vec<ParseError>,
}

impl<T> Iterator for ParseState<T>
where
    T: Clone + core::fmt::Debug,
{
    type Item = T;

    /// Get the next item (if present).
    fn next(&mut self) -> Option<T> {
        if self.index < self.tokens.len() {
            let item = self.tokens.get(self.index).cloned();
            self.index += 1;
            return item;
        }

        None
    }
}

impl<T> ParseState<T>
where
    T: Clone,
{
    pub fn new(tokens: Vec<T>) -> Self {
        Self {
            tokens,
            index: 0,
            errors: vec![],
        }
    }

    /// Peek at the next item.
    pub fn peek(&self) -> Option<T> {
        self.tokens.get(self.index).cloned()
    }

    pub fn peek_reverse(&self) -> Option<T> {
        if self.index > 0 {
            return self.tokens.get(self.index - 1).cloned();
        }

        None
    }

    /// Get the current index.
    pub fn get_index(&self) -> usize {
        self.index
    }

    /// Set the index of this "iterator".
    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn add_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn last_token(&self) -> Option<T> {
        self.tokens.last().cloned()
    }
}

impl<T> From<Vec<T>> for ParseState<T>
where
    T: Clone,
{
    fn from(value: Vec<T>) -> Self {
        Self::new(value)
    }
}
