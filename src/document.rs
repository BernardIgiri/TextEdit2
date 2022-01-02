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
        self.current_revision = 0;
    }
    pub fn open(&mut self, path: std::path::PathBuf) {
        self.reset();
        self.file_path = Some(path.clone());
        let mut input = String::new();
        self.fs.read_to_string(path, &mut input);
        self.history.clear();
        self.history.push(input);
    }
    pub fn save(&mut self, path: std::path::PathBuf) {
        let contents = &self.text();
        self.file_path = Some(path.clone());
        self.fs.write_string(path, contents);
        self.history.remove(0);
        self.history.insert(0, contents.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug, Clone)]
    struct MockFileSystem {
        pub contents: Rc<RefCell<String>>,
    }

    impl Default for MockFileSystem {
        fn default() -> Self {
            Self {
                contents: Rc::new(RefCell::new(String::new())),
            }
        }
    }

    impl FileSystem for MockFileSystem {
        fn read_to_string(&mut self, _path: std::path::PathBuf, contents: &mut String) {
            let data = self.contents.borrow();
            contents.push_str(&data);
        }
        fn write_string(&mut self, _path: std::path::PathBuf, contents: &str) {
            let mut data = self.contents.borrow_mut();
            data.clear();
            data.push_str(contents);
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

    #[test]
    fn test_one_undo() {
        let mut d = TestDocment::default();
        d.update("Mary had a little lamb");
        d.undo();
        assert!(!d.can_undo(), "Undo should not be possible");
        assert!(d.can_redo(), "Redo should be possible");
        assert!(!d.is_dirty());
        assert_eq!("".to_string(), d.original(), "Original text is empty");
        assert_eq!("".to_string(), d.text(), "Updated text is set");
    }

    #[test]
    fn test_third_edit_undo() {
        let mut d = TestDocment::default();
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.undo();
        assert!(d.can_undo(), "Undo should be possible");
        assert!(d.can_redo(), "Redo should be possible");
        assert!(d.is_dirty());
        assert_eq!("".to_string(), d.original(), "Original text is empty");
        assert_eq!(
            "Mary had a little lamb, little lamb".to_string(),
            d.text(),
            "Updated text is set"
        );
    }

    #[test]
    fn test_undo_twice() {
        let mut d = TestDocment::default();
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.undo();
        d.undo();
        assert!(d.can_undo(), "Undo should be possible");
        assert!(d.can_redo(), "Redo should be possible");
        assert!(d.is_dirty());
        assert_eq!("".to_string(), d.original(), "Original text is empty");
        assert_eq!(
            "Mary had a little lamb".to_string(),
            d.text(),
            "Updated text is set"
        );
    }

    #[test]
    fn test_undo_redo() {
        let mut d = TestDocment::default();
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.undo();
        d.undo();
        d.redo();
        assert!(d.can_undo(), "Undo should be possible");
        assert!(d.can_redo(), "Redo should be possible");
        assert!(d.is_dirty());
        assert_eq!("".to_string(), d.original(), "Original text is empty");
        assert_eq!(
            "Mary had a little lamb, little lamb".to_string(),
            d.text(),
            "Updated text is set"
        );
    }

    #[test]
    fn test_undo_redo_3x() {
        let mut d = TestDocment::default();
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.undo();
        d.undo();
        d.undo();
        d.redo();
        d.redo();
        d.redo();
        assert!(d.can_undo(), "Undo should be possible");
        assert!(!d.can_redo(), "Redo should not be possible");
        assert!(d.is_dirty());
        assert_eq!("".to_string(), d.original(), "Original text is empty");
        assert_eq!(
            "Mary had a little lamb, little lamb, little lamb".to_string(),
            d.text(),
            "Updated text is set"
        );
    }

    #[test]
    fn test_reset() {
        let mut d = TestDocment::default();
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.reset();
        assert_eq!(None, d.filepath(), "No file path by default");
        assert_eq!(None, d.filename(), "No filename by default");
        assert!(!d.can_undo(), "No history means no undo by default");
        assert!(!d.can_redo(), "No history means no redo by default");
        assert!(!d.is_dirty());
        assert_eq!("".to_string(), d.original(), "Original text is empty");
        assert_eq!("".to_string(), d.text(), "Default text is empty");
    }

    #[test]
    fn test_open() {
        let fs = MockFileSystem::default();
        let contents = fs.contents.clone();
        let mut d = GenericDocument::new(fs);
        {
            let mut data = contents.borrow_mut();
            data.push_str("There once was an old lady who swallowed a fly.");
        }
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.open(std::path::PathBuf::from("/home/user/sometext.txt"));
        assert_eq!(
            Some(std::path::PathBuf::from("/home/user/sometext.txt")),
            d.filepath(),
            "File path should match opened file."
        );
        assert_eq!(
            Some("sometext.txt".to_string()),
            d.filename(),
            "File name should match opened file"
        );
        assert!(!d.can_undo(), "No history means no undo by default");
        assert!(!d.can_redo(), "No history means no redo by default");
        assert!(!d.is_dirty());
        assert_eq!(
            "There once was an old lady who swallowed a fly.".to_string(),
            d.original(),
            "Original text matches file"
        );
        assert_eq!(
            "There once was an old lady who swallowed a fly.".to_string(),
            d.text(),
            "Text matches file"
        );
    }

    #[test]
    fn test_open_update_undo_all() {
        let fs = MockFileSystem::default();
        let contents = fs.contents.clone();
        let mut d = GenericDocument::new(fs);
        {
            let mut data = contents.borrow_mut();
            data.push_str("There once was an old lady who swallowed a fly.");
        }
        d.open(std::path::PathBuf::from("/home/user/sometext.txt"));
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");

        assert_eq!(
            "Mary had a little lamb, little lamb, little lamb".to_string(),
            d.text(),
            "Text matches update"
        );

        d.undo();
        d.undo();
        d.undo();

        assert_eq!(
            Some(std::path::PathBuf::from("/home/user/sometext.txt")),
            d.filepath(),
            "File path should match opened file."
        );
        assert_eq!(
            Some("sometext.txt".to_string()),
            d.filename(),
            "File name should match opened file"
        );
        assert!(!d.can_undo());
        assert!(d.can_redo());
        assert!(!d.is_dirty());
        assert_eq!(
            "There once was an old lady who swallowed a fly.".to_string(),
            d.original(),
            "Original text matches file"
        );
        assert_eq!(
            "There once was an old lady who swallowed a fly.".to_string(),
            d.text(),
            "Text matches file"
        );
    }

    #[test]
    fn test_open_undo() {
        let fs = MockFileSystem::default();
        let contents = fs.contents.clone();
        let mut d = GenericDocument::new(fs);
        {
            let mut data = contents.borrow_mut();
            data.push_str("There once was an old lady who swallowed a fly.");
        }
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.open(std::path::PathBuf::from("/home/user/sometext.txt"));
        d.update("Let's start over.");
        d.update("Let's start over. THere");
        d.undo();
        assert_eq!(
            Some(std::path::PathBuf::from("/home/user/sometext.txt")),
            d.filepath(),
            "File path should match opened file."
        );
        assert_eq!(
            Some("sometext.txt".to_string()),
            d.filename(),
            "File name should match opened file"
        );
        assert!(d.can_undo());
        assert!(d.can_redo());
        assert!(d.is_dirty());
        assert_eq!(
            "There once was an old lady who swallowed a fly.".to_string(),
            d.original(),
            "Original text matches file"
        );
        assert_eq!(
            "Let's start over.".to_string(),
            d.text(),
            "Text matches file"
        );
    }

    #[test]
    fn test_save() {
        let fs = MockFileSystem::default();
        let contents = fs.contents.clone();
        let mut d = GenericDocument::new(fs);
        {
            let mut data = contents.borrow_mut();
            data.push_str("There once was an old lady who swallowed a fly.");
        }
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.save(std::path::PathBuf::from("/home/user/sometext.txt"));
        assert_eq!(
            Some(std::path::PathBuf::from("/home/user/sometext.txt")),
            d.filepath(),
            "File path should match saved file."
        );
        assert_eq!(
            Some("sometext.txt".to_string()),
            d.filename(),
            "File name should match saved file"
        );
        assert!(d.can_undo());
        assert!(!d.can_redo());
        assert_eq!(
            "Mary had a little lamb, little lamb, little lamb".to_string(),
            d.original(),
            "Original text matches saved data."
        );
        assert_eq!(
            "Mary had a little lamb, little lamb, little lamb".to_string(),
            d.text(),
            "Text matches last update"
        );
        {
            let data = contents.borrow().clone();
            assert_eq!(
                "Mary had a little lamb, little lamb, little lamb".to_string(),
                data,
                "File contents match saved data."
            );
        }
        assert!(!d.is_dirty());
    }

    #[test]
    fn test_save_and_update() {
        let fs = MockFileSystem::default();
        let contents = fs.contents.clone();
        let mut d = GenericDocument::new(fs);
        {
            let mut data = contents.borrow_mut();
            data.push_str("There once was an old lady who swallowed a fly.");
        }
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.save(std::path::PathBuf::from("/home/user/sometext.txt"));
        d.update("Mary had a little lamb, little lamb, lit");
        d.update("Mary had a little lamb, little la");
        d.update("Mary had a little lam");
        d.undo();
        assert_eq!(
            Some(std::path::PathBuf::from("/home/user/sometext.txt")),
            d.filepath(),
            "File path should match saved file."
        );
        assert_eq!(
            Some("sometext.txt".to_string()),
            d.filename(),
            "File name should match saved file"
        );
        assert!(d.can_undo());
        assert!(d.can_redo());
        assert_eq!(
            "Mary had a little lamb, little lamb, little lamb".to_string(),
            d.original(),
            "Original text matches saved data."
        );
        assert_eq!(
            "Mary had a little lamb, little la".to_string(),
            d.text(),
            "Text matches last update"
        );
        {
            let data = contents.borrow().clone();
            assert_eq!(
                "Mary had a little lamb, little lamb, little lamb".to_string(),
                data,
                "File contents match saved data."
            );
        }
        assert!(d.is_dirty());
    }

    #[test]
    fn test_open_update_and_save() {
        let fs = MockFileSystem::default();
        let contents = fs.contents.clone();
        let mut d = GenericDocument::new(fs);
        {
            let mut data = contents.borrow_mut();
            data.push_str("There once was an old lady who swallowed a fly.");
        }
        d.open(std::path::PathBuf::from("/home/user/sometext.txt"));
        d.update("Mary");
        d.update("Mary had");
        d.update("Mary had a");
        d.undo();
        d.save(std::path::PathBuf::from("/home/user/some-other-text.txt"));
        assert_eq!(
            Some(std::path::PathBuf::from("/home/user/some-other-text.txt")),
            d.filepath(),
            "File path should match saved file."
        );
        assert_eq!(
            Some("some-other-text.txt".to_string()),
            d.filename(),
            "File name should match saved file"
        );
        assert!(d.can_undo());
        assert!(d.can_redo());
        assert_eq!(
            "Mary had".to_string(),
            d.original(),
            "Original text matches saved data."
        );
        assert_eq!("Mary had".to_string(), d.text(), "Text matches last update");
        {
            let data = contents.borrow().clone();
            assert_eq!(
                "Mary had".to_string(),
                data,
                "File contents match saved data."
            );
        }
        assert!(!d.is_dirty());
    }
}
