use crate::{
    aliases::Result,
    err::{HarpyError, Kind},
    lexer::{
        span::{Position, Span},
        tokens::{Token, TokenType, Tokenize},
        Lexer,
    },
    parser::{node::NodeId, stmt::stmts::Program, Node},
    source::source_map::FileId,
    tt,
};

#[macro_export]
macro_rules! parse_separated {
    ($parser:expr, $open:tt, $close:tt, $sep:tt, $action:expr) => {{
        $parser.consume::<t!($open)>()?;

        let mut items = Vec::new();

        loop {
            if let tt!($close) | tt!(eof) = $parser.peek()? {
                break;
            }

            items.push($action);

            if let tt!($sep) = $parser.peek()? {
                $parser.consume::<t!($sep)>()?;
            } else {
                break;
            }
        }

        $parser.consume::<t!($close)>()?;

        items
    }};
}

#[macro_export]
macro_rules! parse_sequence {
    ($parser:expr, $open:tt, $close:tt, $parse_expr:expr, $sync_points:expr) => {{
        $parser.consume::<t!($open)>()?;
        let mut items = Vec::new();

        loop {
            if let tt!($close) | tt!(eof) = $parser.peek()? {
                break;
            }

            match $parse_expr {
                Ok(item) => items.push(item),
                Err(e) => $parser.report_error(e, $sync_points)?,
            }
        }

        $parser.consume::<t!($close)>()?;
        items
    }};
}

#[macro_export]
macro_rules! peek_and_consume {
    ($parser:expr, $token:tt) => {
        if let tt!($token) = $parser.peek()? {
            $parser.consume::<t!($token)>()?;
            true
        } else {
            false
        }
    };
}

pub struct Parser<'parser> {
    lexer: Lexer<'parser>,
    next_id: u32,

    pub(super) current: Token,
    pub(super) previous_end: Position,

    errors: Vec<HarpyError>,
}

impl<'parser> Parser<'parser> {
    pub fn new(mut lexer: Lexer<'parser>) -> Self {
        let first_token = lexer.next_token().expect("Failed to fetch initial token");

        Self {
            lexer,
            next_id: 0,
            current: first_token,
            previous_end: Position::default(),
            errors: vec![],
        }
    }

    pub(super) fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        id
    }

    pub(super) fn file_id(&self) -> FileId {
        self.lexer.file_id()
    }

    pub(super) fn peek(&mut self) -> Result<&TokenType> {
        Ok(self.current.kind())
    }

    pub(super) fn consume<T: Tokenize>(&mut self) -> Result<T> {
        let next_token = self.lexer.next_token()?;
        let old_token = std::mem::replace(&mut self.current, next_token);

        self.previous_end = old_token.span().end;
        let token = T::tokenize(old_token)?;

        Ok(token)
    }

    pub(super) fn parse_node<T, F>(&mut self, parsable: F) -> Result<Node<T>>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        let start = self.current.span().start;
        let inner = parsable(self)?;
        let end = self.previous_end;

        Ok(Node {
            id: self.next_id(),
            span: Span::new(start, end, self.lexer.file_id()),
            inner,
        })
    }

    pub(super) fn discard_next(&mut self) -> Result<Token> {
        let next_token = self.lexer.next_token()?;
        let discarded = std::mem::replace(&mut self.current, next_token);
        self.previous_end = discarded.span().end;

        Ok(discarded)
    }

    pub(super) fn report_error(
        &mut self,
        error: HarpyError,
        recovery_points: &[TokenType],
    ) -> Result<()> {
        self.errors.push(error);
        loop {
            let t = self.peek()?;
            if recovery_points.contains(t) {
                break;
            }

            if matches!(t, tt!(eof) | tt!("}")) {
                break;
            }

            if matches!(t, tt!(;)) {
                self.discard_next()?;
                break;
            }

            self.discard_next()?;
        }

        Ok(())
    }

    pub(super) fn unexpected<T>(&mut self, msg: &'static str) -> Result<T> {
        let got = self.current.clone();
        let span = got.span();

        Err(HarpyError::new(
            span,
            Kind::UnexpectedToken { expected: msg, got },
        ))
    }

    pub fn build_ast(mut self) -> std::result::Result<Program, Vec<HarpyError>> {
        let p = match self.parse_program() {
            Ok(p) => p,
            Err(e) => {
                self.errors.push(e);
                return Err(self.errors);
            }
        };

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        Ok(p)
    }
}
