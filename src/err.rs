use crate::{
    aliases::Result,
    color::Color,
    lexer::{
        err::LexerError,
        span::{Position, Span},
    },
    semantic_analyzer::err::SemanticError,
    source::SourceFile,
};

#[derive(Debug)]
pub struct HarpyError {
    kind: HarpyErrorKind,
    span: Span,
}

#[derive(Debug)]
pub enum HarpyErrorKind {
    LexerError(LexerError),
    SemanticError(SemanticError),
    IO(std::io::Error),
}

impl From<std::io::Error> for HarpyError {
    fn from(value: std::io::Error) -> Self {
        Self {
            kind: HarpyErrorKind::IO(value),
            span: Span::default(),
        }
    }
}

impl HarpyError {
    pub fn new(kind: HarpyErrorKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn lexer<S>(err: LexerError, span: Span) -> Result<S> {
        return Err(Self {
            kind: HarpyErrorKind::LexerError(err),
            span,
        });
    }

    pub fn semantic<S>(err: SemanticError, span: Span) -> Result<S> {
        return Err(Self {
            kind: HarpyErrorKind::SemanticError(err),
            span,
        });
    }

    fn highlight_err(line: &str, span: Span) -> String {
        let line = line.trim_end();
        let line_number = span.start.line.to_string();
        let start_col = span.start.column + line_number.len();
        let end_col = span.end.column + line_number.len();
        let highlight_len = std::cmp::max(1, end_col.saturating_sub(start_col));
        let point_line = format!(
            "{}{}{}{}",
            Color::Red,
            " ".repeat(start_col),
            "^".repeat(highlight_len),
            Color::Reset
        );

        format!("{}:{line}\n{point_line}", span.start.line)
    }

    fn format_multiline(lines: Vec<&str>, span: Span) -> String {
        let mut result = String::with_capacity(64);
        let mut lines_iter = lines.iter().peekable();

        let first_line = lines_iter.next().unwrap();
        result += &(Self::highlight_err(
            first_line,
            Span {
                start: span.start,
                end: Position {
                    line: span.start.line,
                    column: first_line.len(),
                    byte: 0,
                },
            },
        ));

        let mut current_line_num = span.start.line + 1;

        while let Some(line) = lines_iter.next() {
            let is_last = lines_iter.peek().is_none();

            let line_span = if is_last {
                Span {
                    start: Position {
                        line: current_line_num,
                        column: 1,
                        byte: 0,
                    },
                    end: Position {
                        line: current_line_num,
                        column: span.end.column,
                        byte: 0,
                    },
                }
            } else {
                Span {
                    start: Position {
                        line: current_line_num,
                        column: 1,
                        byte: 0,
                    },
                    end: Position {
                        line: current_line_num,
                        column: line.len(),
                        byte: 0,
                    },
                }
            };

            result += &Self::highlight_err(line, line_span);
            current_line_num += 1;
        }

        result
    }

    fn format_error(src: &SourceFile, span: Span, err_msg: &str) -> String {
        let start_idx = span.start.line.saturating_sub(1);
        let end_idx = span.end.line.min(src.line_count());

        if start_idx >= src.line_count() {
            return format!(
                "{}{}Error!{} {err_msg} at EOF\n",
                Color::Bold,
                Color::Red,
                Color::Reset
            );
        }

        let lines: Vec<&str> = (start_idx..end_idx)
            .filter_map(|i| src.get_line(i))
            //.map(|s| s.trim_end())
            .collect();

        if lines.is_empty() {
            return format!(
                "{}{}Error!{} {err_msg} at EOF\n",
                Color::Bold,
                Color::Red,
                Color::Reset
            );
        }

        if lines.len() > 1 {
            return Self::format_multiline(lines, span);
        }

        let line = lines[0];

        format!(
            "{}{}Error!{} {err_msg}:\n{} {err_msg}\n\n",
            Color::Bold,
            Color::Red,
            Color::Reset,
            Self::highlight_err(line, span)
        )
    }

    fn io_msg(&self, err: &std::io::Error) -> String {
        format!("IO Error: {:?}", err)
    }

    pub fn show(&self, source: &SourceFile) {
        let msg = match &self.kind {
            HarpyErrorKind::LexerError(e) => e.to_string(),
            HarpyErrorKind::SemanticError(e) => e.to_string(),
            HarpyErrorKind::IO(e) => self.io_msg(e),
        };

        println!("{}", Self::format_error(source, self.span, &msg))
    }
}
