use super::actions::Action;
use super::actions::Action::{OpenFile, SaveFile};
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

    pub fn connect(&self) -> Sender<Action> {
        self.tx
            .as_ref()
            .expect("Attempted to connect before initialized.")
            .clone()
    }

    pub fn update(&mut self, action: Action) {
        match action {
            OpenFile(path) => {
                self.document.open(path);
            }
            SaveFile(path) => {
                self.document.save(path);
            }
        }
    }
}
