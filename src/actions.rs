pub enum Action {
    OpenFile(Option<std::path::PathBuf>),
    SaveFile(std::path::PathBuf),
}
