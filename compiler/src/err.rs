use crate::aliases::Result;
use crate::analyzer::err::AnalyzerError;
use crate::color::Color;
use crate::lexer::{err::LexerError, span::Span};
use crate::source::SourceFile;

#[derive(Debug)]
pub enum HarpyErrorKind {
    Lexer(LexerError),
    IO(std::io::Error),
    Analyzer(AnalyzerError),
}

#[derive(Debug)]
pub struct HarpyError {
    span: Span,
    error: HarpyErrorKind,
}

impl HarpyError {
    pub fn new(error: HarpyErrorKind, span: Span) -> Self {
        Self { error, span }
    }

    pub fn new_analyzer(error: AnalyzerError, span: Span) -> Self {
        Self {
            error: HarpyErrorKind::Analyzer(error),
            span,
        }
    }

    pub fn lexer<T>(error: LexerError, span: Span) -> Result<T> {
        Err(Box::new(Self {
            span,
            error: HarpyErrorKind::Lexer(error),
        }))
    }

    pub fn analyzer<T>(error: AnalyzerError, span: Span) -> Result<T> {
        Err(Box::new(Self {
            span,
            error: HarpyErrorKind::Analyzer(error),
        }))
    }
}

impl From<std::io::Error> for Box<HarpyError> {
    fn from(value: std::io::Error) -> Self {
        Box::new(HarpyError {
            span: Span::default(),
            error: HarpyErrorKind::IO(value),
        })
    }
}

impl HarpyError {
    pub fn print_diagnostic(&self, source: &SourceFile, file_name: &str) {
        let line_num = self.span.start.line;
        let col_num = self.span.start.column;

        let msg = match &self.error {
            HarpyErrorKind::Lexer(e) => format!("{}", e),
            HarpyErrorKind::IO(e) => format!("{}", e),
            HarpyErrorKind::Analyzer(e) => format!("{:?}", e),
        };

        let line_text = source
            .get_line(line_num.saturating_sub(1))
            .unwrap_or("")
            .trim_end();

        let span_len = self.span.end.byte.saturating_sub(self.span.start.byte);
        let squiggle_len = std::cmp::max(1, span_len);

        let safe_squiggle_len = std::cmp::min(
            squiggle_len,
            line_text.len().saturating_sub(col_num.saturating_sub(1)),
        );

        let line_num_str = line_num.to_string();
        let margin = " ".repeat(line_num_str.len());
        let indent = " ".repeat(col_num.saturating_sub(1));
        let squiggles = "^".repeat(safe_squiggle_len);

        println!(
            "{}{}Error: {}{}{}",
            Color::Bold,
            Color::Red,
            Color::Reset,
            Color::Bold,
            msg
        );

        println!(
            "{}  --> {}:{}:{}{}",
            Color::Cyan,
            file_name,
            line_num,
            col_num,
            Color::Reset
        );

        println!("{} {} |{}", Color::Cyan, margin, Color::Reset);

        println!(
            "{} {} |{} {}",
            Color::Cyan,
            line_num_str,
            Color::Reset,
            line_text
        );

        println!(
            "{} {} |{} {}{}{}{}\n",
            Color::Cyan,
            margin,
            Color::Reset,
            indent,
            Color::Red,
            squiggles,
            Color::Reset
        );
    }
}
