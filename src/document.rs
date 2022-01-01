use std::default::Default;
use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

const MAX_UNDO: usize = 100;

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

#[derive(Debug, Clone)]
pub struct GenericDocument<T: FileSystem> {
    history: Vec<String>,
    file_path: Option<std::path::PathBuf>,
    current_revision: usize,
    fs: T,
}

pub type Document = GenericDocument<OSFileSystem>;

impl Default for GenericDocument<OSFileSystem> {
    fn default() -> Self {
        GenericDocument::new(OSFileSystem::default())
    }
}

impl<T: FileSystem> GenericDocument<T> {
    pub fn new(fs: T) -> Self {
        let mut history = Vec::with_capacity(MAX_UNDO);
        history.push(String::new());
        Self {
            history,
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
        self.current_revision > 0
    }
    pub fn can_redo(&self) -> bool {
        self.current_revision < self.history.len() - 1
    }
    pub fn text(&self) -> String {
        match self.history.get(self.current_revision) {
            Some(s) => s.to_string(),
            None => String::new(),
        }
    }
    pub fn original(&self) -> String {
        self.history.get(0).unwrap().to_string()
    }
    pub fn is_dirty(&self) -> bool {
        !self.text().eq(&self.original())
    }
    pub fn undo(&mut self) {
        if self.current_revision > 0 {
            self.current_revision -= 1;
        }
    }
    pub fn redo(&mut self) {
        if !self.history.is_empty() {
            self.current_revision =
                std::cmp::min(self.history.len() - 1, self.current_revision + 1);
        }
    }
    pub fn update(&mut self, value: &str) {
        if self.history.len() >= MAX_UNDO {
            self.history.remove(1);
        }
        self.history.truncate(self.current_revision + 1);
        self.history.push(value.to_string());
        self.current_revision = self.history.len() - 1;
    }
    pub fn reset(&mut self) {
        self.history.clear();
        self.history.push(String::new());
        self.file_path = None;
    }
    pub fn open(&mut self, path: Option<std::path::PathBuf>) {
        self.reset();
        self.file_path = path;
        if let Some(p) = &self.file_path {
            let mut input = String::new();
            self.fs.read_to_string(p.clone(), &mut input);
            self.history.clear();
            self.history.push(input);
        }
    }
    pub fn save(&mut self, path: std::path::PathBuf) {
        self.file_path = Some(path.clone());
        self.fs.write_string(path, &self.text());
        self.history.remove(0);
        self.history.insert(0, self.text())
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
            contents.push_str(path.to_str().unwrap());
        }
        fn write_string(&mut self, path: std::path::PathBuf, contents: &str) {
            self.contents.push_str(contents);
            self.contents.push_str(path.to_str().unwrap());
        }
    }

    impl Default for GenericDocument<MockFileSystem> {
        fn default() -> Self {
            GenericDocument::new(MockFileSystem::default())
        }
    }

    type TestDocment = GenericDocument<MockFileSystem>;

    #[test]
    fn test_default() {
        let d = TestDocment::default();
        assert_eq!(None, d.filepath(), "No file path by default");
        assert_eq!(None, d.filename(), "No filename by default");
        assert!(!d.can_undo(), "No history means no undo by default");
        assert!(!d.can_redo(), "No history means no redo by default");
        assert!(!d.is_dirty());
        assert_eq!("".to_string(), d.original(), "Original text is empty");
        assert_eq!("".to_string(), d.text(), "Default text is empty");
    }

    #[test]
    fn test_one_update() {
        let mut d = TestDocment::default();
        d.update("Mary had a little lamb");
        assert_eq!(None, d.filepath(), "No file path by default");
        assert_eq!(None, d.filename(), "No filename by default");
        assert!(d.can_undo(), "Undo should be possible");
        assert!(!d.can_redo(), "Redo should not be possible");
        assert!(d.is_dirty());
        assert_eq!("".to_string(), d.original(), "Original text is empty");
        assert_eq!(
            "Mary had a little lamb".to_string(),
            d.text(),
            "Updated text is set"
        );
    }

    #[test]
    fn test_two_updates() {
        let mut d = TestDocment::default();
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, whose fleece was white as snow.");
        assert_eq!(None, d.filepath(), "No file path by default");
        assert_eq!(None, d.filename(), "No filename by default");
        assert!(d.can_undo(), "Undo should be possible");
        assert!(!d.can_redo(), "Redo should not be possible");
        assert!(d.is_dirty());
        assert_eq!("".to_string(), d.original(), "Original text is empty");
        assert_eq!(
            "Mary had a little lamb, whose fleece was white as snow.".to_string(),
            d.text(),
            "Updated text is set"
        );
    }

    #[test]
    fn test_many_updates() {
        let mut d = TestDocment::default();
        for _ in 1..1000000 {
            d.update("Mary had a little lamb");
            d.update("Jack jumped over the bean stalk");
        }
        assert_eq!(None, d.filepath(), "No file path by default");
        assert_eq!(None, d.filename(), "No filename by default");
        assert!(d.can_undo(), "Undo should be possible");
        assert!(!d.can_redo(), "Redo should not be possible");
        assert!(d.is_dirty());
        assert_eq!("".to_string(), d.original(), "Original text is empty");
        assert_eq!(
            "Jack jumped over the bean stalk".to_string(),
            d.text(),
            "Updated text is set"
        );
    }
}
