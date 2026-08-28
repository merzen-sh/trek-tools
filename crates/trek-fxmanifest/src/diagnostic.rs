use std::fmt;
use std::fmt::Write as _;
use std::path::Path;

const DEFAULT_CONTEXT_LINES: usize = 1;
const TAB_WIDTH: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn single(line: usize, col: usize) -> Self {
        Self {
            line,
            col,
            end_line: line,
            end_col: col,
        }
    }

    pub fn range(line: usize, col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            line,
            col,
            end_line,
            end_col,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            span: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn at(self, line: usize, col: usize) -> Self {
        self.with_span(Span::single(line, col))
    }

    pub fn range(self, line: usize, col: usize, end_line: usize, end_col: usize) -> Self {
        self.with_span(Span::range(line, col, end_line, end_col))
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.severity, self.message)
    }
}

/// Renders compiler-style codeframes pointing at a location in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Codeframe {
    /// 1-based row of the offending character.
    pub line: usize,
    /// 1-based column of the offending character (start of range).
    pub col: usize,
    /// 1-based column at the end of the error range (exclusive).
    /// When `0`, a single `^` caret is rendered. When `> 0`, tildes `~~~` span the range.
    pub end_col: usize,
    /// How many surrounding lines to include above and below.
    pub context_lines: usize,
}

impl Codeframe {
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            line,
            col,
            end_col: 0,
            context_lines: DEFAULT_CONTEXT_LINES,
        }
    }

    /// Sets the end column for range highlighting. When set (> 0), tildes `~~~`
    /// replace the single caret to span `col..end_col`.
    pub fn end_col(mut self, end_col: usize) -> Self {
        self.end_col = end_col;
        self
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
                "{:>pad$} --> {}:{}:{}",
                "",
                path.display(),
                self.line,
                self.col,
                pad = width
            );
        }
        let _ = writeln!(out, "{:>pad$} |", "", pad = width);

        for n in start..=end {
            let text = lines.get(n - 1).copied().unwrap_or("");
            let _ = writeln!(out, "{n:>pad$} | {text}", pad = width);

            if n == self.line {
                let indicator = self.render_indicator(text);
                let _ = writeln!(out, "{:>pad$} | {indicator}", "", pad = width);
            }
        }

        out
    }

    /// Renders the caret/tilde indicator line for the error line.
    fn render_indicator(&self, text: &str) -> String {
        let visual_col = tab_expanded_col(text, self.col);

        if self.end_col > 0 && self.end_col > self.col {
            let visual_end = tab_expanded_col(text, self.end_col);
            let width = visual_end.saturating_sub(visual_col).max(1);
            format!("{:>visual_col$}{}", "", "~".repeat(width),)
        } else {
            format!("{:>visual_col$}^", "")
        }
    }

    /// Renders a full diagnostic with header, codeframe, and trailing message.
    ///
    /// ```text
    /// [ERROR] SyntaxError: Unclosed string literal in field 'author'
    ///   --> fxmanifest.lua:4:10
    ///    |
    ///  3 |  fx_version 'cerulean'
    ///  4 |  author 'John Doe
    ///    |         ^^^^^^^^^ Unterminated string literal starting at column 10
    ///  5 |  game 'gta5'
    ///    |
    /// ```
    pub fn render_diagnostic(
        &self,
        input: &str,
        path: Option<&Path>,
        severity: Severity,
        category: &str,
        message: &str,
    ) -> String {
        let lines: Vec<&str> = input.lines().collect();
        let total = lines.len().max(1);

        let start = self.line.saturating_sub(self.context_lines).max(1);
        let end = (self.line + self.context_lines).min(total);
        let width = end.to_string().len();

        let mut out = String::new();

        let _ = writeln!(out, "[{}] {}: {}", severity, category, message);

        if let Some(path) = path {
            let _ = writeln!(
                out,
                "{:>pad$} --> {}:{}:{}",
                "",
                path.display(),
                self.line,
                self.col,
                pad = width
            );
        }
        let _ = writeln!(out, "{:>pad$} |", "", pad = width);

        for n in start..=end {
            let text = lines.get(n - 1).copied().unwrap_or("");
            let _ = writeln!(out, "{n:>pad$} | {text}", pad = width);

            if n == self.line {
                let indicator = self.render_indicator(text);
                let _ = writeln!(out, "{:>pad$} | {indicator}", "", pad = width);
            }
        }

        let _ = writeln!(out, "{:>pad$} |", "", pad = width);

        out
    }
}

impl fmt::Display for Codeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render("", None))
    }
}

/// Converts a 1-based column number to its visual offset, expanding tabs to
/// `TAB_WIDTH`-column stops so that caret indicators align correctly in terminals.
fn tab_expanded_col(text: &str, col: usize) -> usize {
    let mut visual = 0usize;
    for (i, c) in text.chars().enumerate() {
        if i + 1 >= col {
            break;
        }
        if c == '\t' {
            visual = (visual / TAB_WIDTH + 1) * TAB_WIDTH;
        } else {
            visual += 1;
        }
    }
    visual
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

    #[test]
    fn range_indicator_uses_tildes() {
        let frame = Codeframe::new(1, 8)
            .end_col(15)
            .render("version '1.0", None);
        // col 8 -> visual 7, end_col 15 -> visual 12, width = 5
        assert!(frame.contains("~~~~~"), "{frame}");
        assert!(!frame.contains('^'), "{frame}");
    }

    #[test]
    fn range_indicator_single_char() {
        let frame = Codeframe::new(1, 1).end_col(2).render("ab", None);
        assert!(frame.contains('~'), "{frame}");
        assert!(!frame.contains('^'), "{frame}");
    }

    #[test]
    fn tab_characters_expand_in_indicator() {
        let frame = Codeframe::new(1, 5).render("\tval", None);
        // Tab occupies cols 1-4 (visual offset 0..4), col 5 = 'v' at visual offset 4
        assert!(frame.contains("    ^"), "{frame}");
    }

    #[test]
    fn render_diagnostic_includes_header() {
        let frame = Codeframe::new(1, 1);
        let out = frame.render_diagnostic(
            "hello\nworld",
            Some(Path::new("test.lua")),
            Severity::Error,
            "SyntaxError",
            "something went wrong",
        );
        assert!(
            out.starts_with("[error] SyntaxError: something went wrong"),
            "{out}"
        );
        assert!(out.contains("--> test.lua:1:1"), "{out}");
        assert!(out.contains("1 | hello"), "{out}");
        assert!(out.contains("2 | world"), "{out}");
    }
}
