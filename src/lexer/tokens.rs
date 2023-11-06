/// Struct for iterating over a vector of tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokens<T> {
    tokens: Vec<T>,
    index: usize,
}

impl<T> Iterator for Tokens<T>
where
    T: Clone,
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

impl<T> Tokens<T>
where
    T: Clone,
{
    pub fn new(tokens: Vec<T>) -> Self {
        Self { tokens, index: 0 }
    }

    /// Peek at the next item.
    pub fn peek(&mut self) -> Option<T> {
        return self.tokens.get(self.index).cloned();
    }

    /// Get the current index.
    pub fn get_index(&self) -> usize {
        self.index
    }

    /// Set the index of this "iterator".
    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}

impl<T> From<Vec<T>> for Tokens<T>
where
    T: Clone,
{
    fn from(value: Vec<T>) -> Self {
        Self::new(value)
    }
}
