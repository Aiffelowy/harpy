use crate::{aliases::Result, source::SourceFile};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileId(pub usize);

#[derive(Default)]
pub struct SourceMap {
    pub files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn load_source(&mut self, filename: String) -> Result<FileId> {
        let id = FileId(self.files.len());
        let source = SourceFile::new(filename, id)?;
        self.files.push(source);
        Ok(id)
    }

    pub fn get_source(&self, id: FileId) -> &SourceFile {
        &self.files[id.0]
    }
}
