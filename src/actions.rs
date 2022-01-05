pub enum Action {
    OpenFile(Option<std::path::PathBuf>),
    SaveFile(std::path::PathBuf),
    DocumentChanged(String),
    FileOpenFinished(IOResult),
    FileSaveFinished(IOResult),
}

#[derive(Debug, Clone)]
pub enum Err {
    IOError(),
    UnknownError(),
}

pub type IOResult = Result<(std::path::PathBuf, String), Err>;
