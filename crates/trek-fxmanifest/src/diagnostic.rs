use std::fmt;
use std::fmt::Write as _;
use std::path::Path;

const DEFAULT_CONTEXT_LINES: usize = 1;

/// Renders compiler-style codeframes pointing at a location in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Codeframe {
    /// 1-based row of the offending character.
    pub line: usize,
    /// 1-based column of the offending character.
    pub col: usize,
    /// How many surrounding lines to include above and below.
    pub context_lines: usize,
}

impl Codeframe {
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            line,
            col,
            context_lines: DEFAULT_CONTEXT_LINES,
        }
    }

    /// Sets how many surrounding lines are included above and below.
    pub fn context_lines(mut self, count: usize) -> Self {
        self.context_lines = count;
        self
    }

    /// Renders a codeframe such as:
    ///
    /// ```text
    ///    --> fxmanifest.lua:3:11
    ///     |
    ///   2 | game 'gta5'
    ///   3 | version = ~
    ///     |           ^
    /// ```
    ///
    /// The returned text is unstyled; callers may apply colors afterwards.
    pub fn render(&self, input: &str, path: Option<&Path>) -> String {
        let lines: Vec<&str> = input.lines().collect();
        let total = lines.len().max(1);

        let start = self.line.saturating_sub(self.context_lines).max(1);
        let end = (self.line + self.context_lines).min(total);
        let width = end.to_string().len();

        let mut out = String::new();

        if let Some(path) = path {
            let _ = writeln!(
                out,
                "{:>width$} --> {}:{}:{}",
                "",
                path.display(),
                self.line,
                self.col,
                width = width
            );
        }
        let _ = writeln!(out, "{:>width$} |", "", width = width);

        for n in start..=end {
            let text = lines.get(n - 1).copied().unwrap_or("");
            let _ = writeln!(out, "{n:>width$} | {text}");

            if n == self.line {
                let caret_offset = self.col.saturating_sub(1).min(text.chars().count());
                let _ = writeln!(out, "{:>width$} | {:caret_offset$}^", "", "", width = width);
            }
        }

        out
    }
}

impl fmt::Display for Codeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render("", None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "fx_version 'cerulean'\ngame 'gta5'\nversion = ~\nauthor 'x'\n";

    #[test]
    fn renders_location_header_and_caret() {
        let frame = Codeframe::new(3, 11).render(SOURCE, Some(Path::new("fxmanifest.lua")));

        let expected = concat!(
            "  --> fxmanifest.lua:3:11\n",
            "  |\n",
            "2 | game 'gta5'\n",
            "3 | version = ~\n",
            "  |           ^\n",
            "4 | author 'x'\n"
        );
        assert_eq!(frame, expected);
    }

    #[test]
    fn clamps_caret_beyond_line_length() {
        let frame = Codeframe::new(1, 999).render("abc", None);
        assert_eq!(frame, "  |\n1 | abc\n  |    ^\n");
    }

    #[test]
    fn zero_context_renders_single_line() {
        let frame = Codeframe::new(2, 2)
            .context_lines(0)
            .render("a\nb\nc", None);
        assert_eq!(frame, "  |\n2 | b\n  |  ^\n");
    }

    #[test]
    fn line_one_has_no_context_above() {
        let frame = Codeframe::new(1, 1).render("a\nb\nc", None);
        assert!(frame.starts_with("  |\n1 | a\n"));
        assert!(!frame.contains('c'), "{frame}");
    }
}
