use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct FormatterContext {
    pub output: String,
    indent_level: usize,
    indent_string: String,
}

impl FormatterContext {
    pub fn new() -> Self {
        Self::with_indent_size(4)
    }
    
    pub fn with_indent_size(indent_size: usize) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_string: " ".repeat(indent_size),
        }
    }
    
    pub fn write(&mut self, text: &str) -> Result<(), std::fmt::Error> {
        self.output.write_str(text)
    }
    
    pub fn write_char(&mut self, c: char) -> Result<(), std::fmt::Error> {
        self.output.write_char(c)
    }
    
    pub fn write_indent(&mut self) -> Result<(), std::fmt::Error> {
        let indent_string = self.indent_string.clone();
        for _ in 0..self.indent_level {
            self.write(&indent_string)?;
        }
        Ok(())
    }
    
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }
    
    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
    
    pub fn with_indent<F>(&mut self, f: F) -> Result<(), std::fmt::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), std::fmt::Error>,
    {
        self.indent();
        let result = f(self);
        self.dedent();
        result
    }
    
    pub fn write_newline(&mut self) -> Result<(), std::fmt::Error> {
        self.write("\n")
    }
    
    pub fn write_indented_line(&mut self, text: &str) -> Result<(), std::fmt::Error> {
        self.write_indent()?;
        self.write(text)?;
        self.write_newline()
    }
    
    pub fn write_separated<T, F>(&mut self, items: &[T], separator: &str, mut formatter: F) -> Result<(), std::fmt::Error>
    where
        F: FnMut(&mut Self, &T) -> Result<(), std::fmt::Error>,
    {
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                self.write(separator)?;
            }
            formatter(self, item)?;
        }
        Ok(())
    }
}

impl Default for FormatterContext {
    fn default() -> Self {
        Self::new()
    }
}