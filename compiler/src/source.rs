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
            .unwrap_or_else(|| self.text.len());
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
