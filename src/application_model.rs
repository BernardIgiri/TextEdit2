use super::actions::Action;
use super::actions::Action::*;
use super::document::Document;
use crate::glib::Sender;

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

    pub fn update(&mut self, action: Action) -> bool {
        match action {
            OpenFile(Some(path)) => {
                self.document.open(path);
                true
            }
            OpenFile(None) => {
                self.document.reset();
                true
            }
            SaveFile(path) => {
                self.document.save(path);
                false
            }
            DocumentChanged(value) => {
                self.document.update(value.as_str());
                false
            }
        }
    }
}
