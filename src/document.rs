use std::default::Default;
use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub trait FileSystem: Debug + Clone {
    fn read_to_string(&mut self, path: std::path::PathBuf, contents: &mut String);
    fn write_string(&mut self, path: std::path::PathBuf, contents: &str);
}

#[derive(Default, Debug, Clone)]
pub struct OSFileSystem {}

impl FileSystem for OSFileSystem {
    fn read_to_string(&mut self, path: std::path::PathBuf, contents: &mut String) {
        let file = File::open(path).expect("Couldn't open file");
        let mut reader = BufReader::new(file);
        reader.read_to_string(contents).expect("Couldn't read file");
    }
    fn write_string(&mut self, path: std::path::PathBuf, contents: &str) {
        let mut file = File::create(path).expect("Couldn't write file");
        file.write_all(contents.as_bytes())
            .expect("Couldn't write file");
    }
}

#[derive(Default, Debug, Clone)]
pub struct GenericDocument<T: FileSystem> {
    original: String,
    history: Vec<String>,
    file_path: Option<std::path::PathBuf>,
    current_revision: usize,
    fs: T,
}

pub type Document = GenericDocument<OSFileSystem>;

impl<T: FileSystem> GenericDocument<T> {
    pub fn new(fs: T) -> Self {
        Self {
            original: "".into(),
            history: Vec::new(),
            file_path: None,
            current_revision: 0,
            fs,
        }
    }
    pub fn filepath(&self) -> Option<std::path::PathBuf> {
        self.file_path.clone()
    }
    pub fn filename(&self) -> Option<String> {
        match &self.file_path {
            None => None,
            Some(path) => match path.file_name().unwrap().to_os_string().into_string() {
                Ok(s) => Some(s),
                _ => None,
            },
        }
    }
    pub fn can_undo(&self) -> bool {
        self.history.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        self.history.len() - 1 > self.current_revision
    }
    pub fn text(&self) -> String {
        match self.history.get(self.current_revision) {
            Some(s) => s.to_string(),
            None => String::new(),
        }
    }
    pub fn is_dirty(&self) -> bool {
        !self.history.is_empty() && self.text().eq(&self.original)
    }
    pub fn undo(&mut self) {
        self.current_revision = std::cmp::max(0, self.current_revision - 1);
    }
    pub fn redo(&mut self) {
        self.current_revision = std::cmp::min(self.history.len() - 1, self.current_revision + 1);
    }
    pub fn update(&mut self, value: &str) {
        self.history.truncate(self.current_revision);
        self.history.push(value.to_string());
        self.current_revision += 1;
    }
    pub fn reset(&mut self) {
        self.original = "".into();
        self.history.clear();
        self.file_path = None;
    }
    pub fn open(&mut self, path: Option<std::path::PathBuf>) {
        self.reset();
        self.file_path = path;
        if let Some(p) = &self.file_path {
            self.fs.read_to_string(p.clone(), &mut self.original);
        }
    }
    pub fn save(&mut self, path: std::path::PathBuf) {
        self.file_path = Some(path.clone());
        self.original = self.text();
        self.fs.write_string(path, &self.original);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, Clone)]
    struct MockFileSystem {
        pub contents: String,
    }

    impl FileSystem for MockFileSystem {
        fn read_to_string(&mut self, path: std::path::PathBuf, contents: &mut String) {
            contents.push_str(&self.contents);
            contents.push_str(&path.to_str().unwrap());
        }
        fn write_string(&mut self, path: std::path::PathBuf, contents: &str) {
            self.contents.push_str(contents);
            self.contents.push_str(&path.to_str().unwrap());
        }
    }

    type TestDocment = GenericDocument<MockFileSystem>;

    #[test]
    fn test_default() {
        let d = TestDocment::default();
        assert_eq!(None, d.filepath());
        assert_eq!(None, d.filename());
        assert!(!d.can_undo());
        assert!(!d.can_redo());
        assert_eq!("", d.text());
    }
}
