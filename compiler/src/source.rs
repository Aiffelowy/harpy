use std::{
    io::BufRead,
    ops::{Index, Range, RangeFrom, RangeTo},
};

use crate::aliases::Result;

#[derive(Debug)]
pub struct SourceFile {
    pub text: String,
    pub line_starts: Vec<usize>, // byte offsets
}

impl SourceFile {
    pub fn new<Reader: BufRead>(mut reader: Reader) -> Result<Self> {
        let mut text = String::new();
        reader.read_to_string(&mut text)?;
        let mut line_starts = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Ok(Self { text, line_starts })
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn get_line(&self, line_idx: usize) -> Option<&str> {
        let start = *self.line_starts.get(line_idx)?;
        let end = self
            .line_starts
            .get(line_idx + 1)
            .copied()
            .unwrap_or(self.text.len());
        Some(&self.text[start..end])
    }
}

impl Index<Range<usize>> for SourceFile {
    type Output = str;
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.text[index]
    }
}

impl Index<RangeFrom<usize>> for SourceFile {
    type Output = str;
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        &self.text[index]
    }
}

impl Index<RangeTo<usize>> for SourceFile {
    type Output = str;
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        &self.text[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_source_file_creation_empty() {
        let reader = Cursor::new("");
        let source = SourceFile::new(reader).unwrap();
        assert_eq!(source.text, "");
        assert_eq!(source.line_starts, vec![0]);
        assert_eq!(source.line_count(), 1);
    }

    #[test]
    fn test_source_file_creation_single_line() {
        let reader = Cursor::new("hello world");
        let source = SourceFile::new(reader).unwrap();
        assert_eq!(source.text, "hello world");
        assert_eq!(source.line_starts, vec![0]);
        assert_eq!(source.line_count(), 1);
    }

    #[test]
    fn test_source_file_creation_multiple_lines() {
        let reader = Cursor::new("line 1\nline 2\nline 3");
        let source = SourceFile::new(reader).unwrap();
        assert_eq!(source.text, "line 1\nline 2\nline 3");
        assert_eq!(source.line_starts, vec![0, 7, 14]);
        assert_eq!(source.line_count(), 3);
    }

    #[test]
    fn test_source_file_get_line() {
        let reader = Cursor::new("first\nsecond\nthird");
        let source = SourceFile::new(reader).unwrap();
        assert_eq!(source.get_line(0), Some("first\n"));
        assert_eq!(source.get_line(1), Some("second\n"));
        assert_eq!(source.get_line(2), Some("third"));
        assert_eq!(source.get_line(3), None);
    }

    #[test]
    fn test_source_file_get_line_with_trailing_newline() {
        let reader = Cursor::new("line1\nline2\n");
        let source = SourceFile::new(reader).unwrap();
        assert_eq!(source.get_line(0), Some("line1\n"));
        assert_eq!(source.get_line(1), Some("line2\n"));
        assert_eq!(source.get_line(2), Some(""));
    }

    #[test]
    fn test_source_file_indexing_range() {
        let reader = Cursor::new("hello world");
        let source = SourceFile::new(reader).unwrap();
        assert_eq!(&source[0..5], "hello");
        assert_eq!(&source[6..11], "world");
    }

    #[test]
    fn test_source_file_indexing_range_from() {
        let reader = Cursor::new("hello world");
        let source = SourceFile::new(reader).unwrap();
        assert_eq!(&source[6..], "world");
        assert_eq!(&source[0..], "hello world");
    }

    #[test]
    fn test_source_file_indexing_range_to() {
        let reader = Cursor::new("hello world");
        let source = SourceFile::new(reader).unwrap();
        assert_eq!(&source[..5], "hello");
        assert_eq!(&source[..11], "hello world");
    }
}
