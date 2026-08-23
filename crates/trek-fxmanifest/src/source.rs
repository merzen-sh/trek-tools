use std::fmt;
use std::path::Path;

/// The raw manifest input managed by the first stage of the pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source<'a> {
    input: &'a str,
}

impl<'a> Source<'a> {
    /// Wraps raw manifest text.
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }

    /// Reads manifest text from a file on disk.
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    pub fn as_str(&self) -> &'a str {
        self.input
    }

    pub fn len(&self) -> usize {
        self.input.len()
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }
}

impl fmt::Display for Source<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.input)
    }
}
