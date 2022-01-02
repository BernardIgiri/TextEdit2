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

#[derive(Debug, Clone)]
pub struct GenericDocument<T: FileSystem> {
    original: String,
    text: String,
    file_path: Option<std::path::PathBuf>,
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
        Self {
            original: String::new(),
            text: String::new(),
            file_path: None,
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
    pub fn text(&self) -> &String {
        &self.text
    }
    pub fn original(&self) -> &String {
        &self.original
    }
    pub fn is_dirty(&self) -> bool {
        !self.text().eq(self.original())
    }
    pub fn update(&mut self, value: &str) {
        self.text.clear();
        self.text.push_str(value);
    }
    pub fn reset(&mut self) {
        self.text.clear();
        self.original.clear();
        self.file_path = None;
    }
    pub fn open(&mut self, path: std::path::PathBuf) {
        self.reset();
        self.file_path = Some(path.clone());
        self.fs.read_to_string(path, &mut self.original);
        self.text.clear();
        self.text.push_str(self.original.as_str());
    }
    pub fn save(&mut self, path: std::path::PathBuf) {
        self.file_path = Some(path.clone());
        self.fs.write_string(path, self.text.as_str());
        self.original.clear();
        self.original.push_str(self.text.as_str());
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
        assert!(!d.is_dirty());
        let original = String::new();
        let text = String::new();
        assert_eq!(&original, d.original(), "Original text is empty");
        assert_eq!(&text, d.text(), "Default text is empty");
    }

    #[test]
    fn test_one_update() {
        let mut d = TestDocment::default();
        d.update("Mary had a little lamb");
        assert_eq!(None, d.filepath(), "No file path by default");
        assert_eq!(None, d.filename(), "No filename by default");
        assert!(d.is_dirty());
        let original = String::new();
        let text = String::from("Mary had a little lamb");
        assert_eq!(&original, d.original(), "Original text is empty");
        assert_eq!(&text, d.text(), "Updated text is set");
    }

    #[test]
    fn test_two_updates() {
        let mut d = TestDocment::default();
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, whose fleece was white as snow.");
        assert!(d.is_dirty());
        let original = String::new();
        let text = String::from("Mary had a little lamb, whose fleece was white as snow.");
        assert_eq!(&original, d.original(), "Original text is empty");
        assert_eq!(&text, d.text(), "Updated text is set");
    }

    #[test]
    fn test_many_updates() {
        let mut d = TestDocment::default();
        for _ in 1..1000000 {
            d.update("Mary had a little lamb");
            d.update("Jack jumped over the bean stalk");
        }
        assert!(d.is_dirty());
        let original = String::new();
        let text = String::from("Jack jumped over the bean stalk");
        assert_eq!(&original, d.original(), "Original text is empty");
        assert_eq!(&text, d.text(), "Updated text is set");
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
        assert!(!d.is_dirty());
        let original = String::new();
        let text = String::new();
        assert_eq!(&original, d.original(), "Original text is empty");
        assert_eq!(&text, d.text(), "Default text is empty");
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
        assert!(!d.is_dirty());
        let original = String::from("There once was an old lady who swallowed a fly.");
        let text = String::from("There once was an old lady who swallowed a fly.");
        assert_eq!(&original, d.original(), "Original text matches file");
        assert_eq!(&text, d.text(), "Text matches file");
    }

    #[test]
    fn test_open_and_update() {
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

        let original = String::from("There once was an old lady who swallowed a fly.");
        let text = String::from("Mary had a little lamb, little lamb, little lamb");

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
        assert!(d.is_dirty());
        assert_eq!(&original, d.original(), "Original text matches file");
        assert_eq!(&text, d.text(), "Text matches update");
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
        let text = String::from("Mary had a little lamb, little lamb, little lamb");
        assert_eq!(&text, d.original(), "Original text matches saved data.");
        assert_eq!(&text, d.text(), "Text matches last update");
        {
            let data = contents.borrow().clone();
            assert_eq!(&text, &data, "File contents match saved data.");
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
        let original = String::from("Mary had a little lamb, little lamb, little lamb");
        let text = String::from("Mary had a little lamb, little la");
        assert_eq!(&original, d.original(), "Original text matches saved data.");
        assert_eq!(&text, d.text(), "Text matches last update");
        {
            let data = contents.borrow().clone();
            assert_eq!(&original, &data, "File contents match saved data.");
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
        let original = String::from("Mary had");
        let text = String::from("Mary had");
        assert_eq!(&original, d.original(), "Original text matches saved data.");
        assert_eq!(&text, d.text(), "Text matches last update");
        {
            let data = contents.borrow().clone();
            assert_eq!(&text, &data, "File contents match saved data.");
        }
        assert!(!d.is_dirty());
    }
}
