use std::default::Default;
use std::fmt::Debug;

#[derive(Debug, Default, Clone)]
pub struct Document {
    original: String,
    text: String,
    file_path: Option<std::path::PathBuf>,
}

impl Document {
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
    pub fn modified(&self) -> bool {
        !self.text().eq(self.original())
    }
    pub fn update(&mut self, value: &str) {
        self.text = value.to_string()
    }
    pub fn reset(&mut self) {
        self.text.clear();
        self.original.clear();
        self.file_path = None;
    }
    pub fn open(&mut self, path: std::path::PathBuf, contents: String) {
        self.file_path = Some(path);
        self.original = contents.clone();
        self.text = contents;
    }
    pub fn save(&mut self, path: std::path::PathBuf, contents: String) {
        self.file_path = Some(path);
        self.original = contents;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let d = Document::default();
        assert_eq!(None, d.filepath(), "No file path by default");
        assert_eq!(None, d.filename(), "No filename by default");
        assert!(!d.modified());
        let original = String::new();
        let text = String::new();
        assert_eq!(&original, d.original(), "Original text is empty");
        assert_eq!(&text, d.text(), "Default text is empty");
    }

    #[test]
    fn test_one_update() {
        let mut d = Document::default();
        d.update("Mary had a little lamb");
        assert_eq!(None, d.filepath(), "No file path by default");
        assert_eq!(None, d.filename(), "No filename by default");
        assert!(d.modified());
        let original = String::new();
        let text = String::from("Mary had a little lamb");
        assert_eq!(&original, d.original(), "Original text is empty");
        assert_eq!(&text, d.text(), "Updated text is set");
    }

    #[test]
    fn test_two_updates() {
        let mut d = Document::default();
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, whose fleece was white as snow.");
        assert!(d.modified());
        let original = String::new();
        let text = String::from("Mary had a little lamb, whose fleece was white as snow.");
        assert_eq!(&original, d.original(), "Original text is empty");
        assert_eq!(&text, d.text(), "Updated text is set");
    }

    #[test]
    fn test_many_updates() {
        let mut d = Document::default();
        for _ in 1..1000000 {
            d.update("Mary had a little lamb");
            d.update("Jack jumped over the bean stalk");
        }
        assert!(d.modified());
        let original = String::new();
        let text = String::from("Jack jumped over the bean stalk");
        assert_eq!(&original, d.original(), "Original text is empty");
        assert_eq!(&text, d.text(), "Updated text is set");
    }

    #[test]
    fn test_reset() {
        let mut d = Document::default();
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.reset();
        assert_eq!(None, d.filepath(), "No file path by default");
        assert_eq!(None, d.filename(), "No filename by default");
        assert!(!d.modified());
        let original = String::new();
        let text = String::new();
        assert_eq!(&original, d.original(), "Original text is empty");
        assert_eq!(&text, d.text(), "Default text is empty");
    }

    #[test]
    fn test_open() {
        let mut d = Document::default();
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.open(
            std::path::PathBuf::from("/home/user/sometext.txt"),
            "There once was an old lady who swallowed a fly.".into(),
        );
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
        assert!(!d.modified());
        let original = String::from("There once was an old lady who swallowed a fly.");
        let text = String::from("There once was an old lady who swallowed a fly.");
        assert_eq!(&original, d.original(), "Original text matches file");
        assert_eq!(&text, d.text(), "Text matches file");
    }

    #[test]
    fn test_save() {
        let mut d = Document::default();
        d.update("Mary had a little lamb");
        d.update("Mary had a little lamb, little lamb");
        d.update("Mary had a little lamb, little lamb, little lamb");
        d.save(
            std::path::PathBuf::from("/home/user/sometext.txt"),
            "Mary had a little lamb, little lamb, little lamb".into(),
        );
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
        assert!(!d.modified());
    }
}
