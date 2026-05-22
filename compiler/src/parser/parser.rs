use crate::{
    aliases::Result,
    err::{HarpyError, Kind},
    lexer::{
        span::{Position, Span},
        tokens::{Token, TokenType, Tokenize},
        Lexer,
    },
    parser::{node::NodeId, stmt::stmts::Program, Node},
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

    pub(super) previous_end: Position,

    errors: Vec<HarpyError>,
}

impl<'parser> Parser<'parser> {
    pub fn new(lexer: Lexer<'parser>) -> Self {
        Self {
            lexer,
            next_id: 0,
            previous_end: Position::default(),
            errors: vec![],
        }
    }

    pub(super) fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        id
    }

    pub(super) fn peek(&mut self) -> Result<&TokenType> {
        self.lexer.peek()
    }

    pub(super) fn consume<T: Tokenize>(&mut self) -> Result<T> {
        let token = T::tokenize(&mut self.lexer)?;
        self.previous_end = token.span().end;
        Ok(token)
    }

    pub(super) fn parse_node<T, F>(&mut self, parsable: F) -> Result<Node<T>>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        let start = self.lexer.position();
        let inner = parsable(self)?;
        let end = self.previous_end;

        Ok(Node {
            id: self.next_id(),
            span: Span::new(start, end),
            inner,
        })
    }

    pub(super) fn discard_next(&mut self) -> Result<Token> {
        self.lexer.next_token()
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
        let t = self.lexer.next_token()?;
        let span = t.span();

        Err(HarpyError::new(
            span,
            Kind::UnexpectedToken {
                expected: msg,
                got: t,
            },
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
