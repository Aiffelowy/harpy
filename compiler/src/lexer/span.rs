use crate::source::source_map::FileId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub byte: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            byte: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub file_id: FileId,
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position, file_id: FileId) -> Self {
        Self {
            start,
            end,
            file_id,
        }
    }

    pub fn dummy() -> Self {
        Self {
            start: Position::default(),
            end: Position::default(),
            file_id: FileId(0),
        }
    }

    pub fn merge(self, other: Span) -> Self {
        Span::new(self.start, other.end, self.file_id)
    }
}
