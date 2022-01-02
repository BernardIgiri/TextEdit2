use super::actions::Action;
use super::actions::Action::*;
use super::document::Document;
use crate::glib::Sender;

#[derive(Debug, Default, Clone)]
pub struct Changes {
    pub filename: bool,
    pub text: bool,
}

impl Changes {
    pub fn new(filename: bool, text: bool) -> Self {
        Self { filename, text }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ApplicationModel {
    document: Document,
    tx: Option<Sender<Action>>,
}

impl ApplicationModel {
    pub fn new() -> Self {
        Self {
            document: Document::default(),
            tx: None,
        }
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
                self.document.open(path);
                Changes::new(true, true)
            }
            OpenFile(None) => {
                self.document.reset();
                Changes::new(true, true)
            }
            SaveFile(path) => {
                self.document.save(path);
                Changes::new(true, false)
            }
            DocumentChanged(value) => {
                self.document.update(value.as_str());
                Changes::new(false, false)
            }
        }
    }
}
