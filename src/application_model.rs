use super::actions::Action::*;
use super::actions::{Action, Err, IOResult};
use super::document::Document;
use crate::glib::Sender;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::thread;

#[derive(Debug, Clone)]
pub enum StatusMessage {
    None,
    OpeningFile,
    SavingFile,
    FileSaveFinished(Result<(), Err>),
    FileOpenFinished(Result<(), Err>),
}

impl Default for StatusMessage {
    fn default() -> Self {
        StatusMessage::None
    }
}

#[derive(Debug, Default, Clone)]
pub struct Changes {
    pub filename: bool,
    pub text: bool,
    pub status_message: bool,
}

impl Changes {
    pub fn new(filename: bool, text: bool, status_message: bool) -> Self {
        Self {
            filename,
            text,
            status_message,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ApplicationModel {
    document: Document,
    status_message: StatusMessage,
    tx: Option<Sender<Action>>,
}

impl ApplicationModel {
    pub fn new() -> Self {
        Self {
            document: Document::default(),
            status_message: StatusMessage::default(),
            tx: None,
        }
    }

    pub fn status_message(&self) -> &StatusMessage {
        &self.status_message
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn transmit(&mut self, tx: Sender<Action>) {
        self.tx = Some(tx);
    }

    pub fn send(&self, action: Action) {
        self.tx.as_ref().unwrap().send(action).ok();
    }

    pub fn update(&mut self, action: Action) -> Changes {
        match action {
            OpenFile(Some(path)) => {
                let tx = self.tx.as_ref().unwrap().clone();
                thread::spawn(move || {
                    let mut contents = String::new();
                    let r = match FileSystem::read_to_string(path.clone(), &mut contents) {
                        Ok(()) => IOResult::Ok((path, contents)),
                        Err(_) => IOResult::Err(Err::IOError()),
                    };
                    tx.send(FileOpenFinished(r)).ok()
                });
                self.status_message = StatusMessage::OpeningFile;
                Changes::new(false, false, true)
            }
            OpenFile(None) => {
                self.document.reset();
                self.status_message = StatusMessage::OpeningFile;
                Changes::new(false, false, true)
            }
            SaveFile(path) => {
                let tx = self.tx.as_ref().unwrap().clone();
                let contents = self.document.text().clone();
                thread::spawn(move || {
                    let r = match FileSystem::write_string(path.clone(), &contents) {
                        Ok(()) => IOResult::Ok((path, contents)),
                        Err(_) => IOResult::Err(Err::IOError()),
                    };
                    tx.send(FileSaveFinished(r)).ok()
                });
                self.status_message = StatusMessage::SavingFile;
                Changes::new(false, false, true)
            }
            DocumentChanged(value) => {
                self.document.update(value.as_str());
                Changes::new(false, false, false)
            }
            FileOpenFinished(Ok((path, contents))) => {
                self.document.open(path, contents);
                self.status_message = StatusMessage::FileOpenFinished(Ok(()));
                Changes::new(true, true, true)
            }
            FileSaveFinished(Ok((path, contents))) => {
                self.document.save(path, contents);
                self.status_message = StatusMessage::FileSaveFinished(Ok(()));
                Changes::new(true, false, true)
            }
            FileOpenFinished(Err(e)) => {
                self.status_message = StatusMessage::FileOpenFinished(Err(e));
                Changes::new(false, false, true)
            }
            FileSaveFinished(Err(e)) => {
                self.status_message = StatusMessage::FileSaveFinished(Err(e));
                Changes::new(false, false, true)
            }
        }
    }
}

struct FileSystem {}

impl FileSystem {
    fn read_to_string(path: std::path::PathBuf, contents: &mut String) -> io::Result<()> {
        let file = File::open(path)?;
        let mut reader = io::BufReader::new(file);
        reader.read_to_string(contents)?;
        Ok(())
    }
    fn write_string(path: std::path::PathBuf, contents: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }
}
