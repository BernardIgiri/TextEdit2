use gettextrs::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use log::debug;

use super::actions::Action;
use super::actions::Action::DocumentChanged;
use crate::glib::Sender;

use super::application_model::{ApplicationModel, Changes, StatusMessage};
use crate::application::Application;
use crate::config::{APP_ID, PROFILE};

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/bernardigiri/TextEdit2/ui/window.ui")]
    pub struct ApplicationWindow {
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub modified: TemplateChild<gtk::Label>,
        #[template_child]
        pub headerbar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub bodytext: TemplateChild<gtk::TextView>,
        pub settings: gio::Settings,
        #[template_child]
        pub save_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub open_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub status_bar: TemplateChild<gtk::Label>,
    }

    impl Default for ApplicationWindow {
        fn default() -> Self {
            Self {
                title: TemplateChild::default(),
                modified: TemplateChild::default(),
                headerbar: TemplateChild::default(),
                bodytext: TemplateChild::default(),
                save_button: TemplateChild::default(),
                open_button: TemplateChild::default(),
                status_bar: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ApplicationWindow {
        const NAME: &'static str = "ApplicationWindow";
        type Type = super::ApplicationWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ApplicationWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            // Load latest window state
            obj.load_window_size();
        }
    }

    impl WidgetImpl for ApplicationWindow {}
    impl WindowImpl for ApplicationWindow {
        // Save window state on delete event
        fn close_request(&self, window: &Self::Type) -> gtk::Inhibit {
            if let Err(err) = window.save_window_size() {
                log::warn!("Failed to save window state, {}", &err);
            }

            // Pass close request on to the parent
            self.parent_close_request(window)
        }
    }

    impl ApplicationWindowImpl for ApplicationWindow {}
}

glib::wrapper! {
    pub struct ApplicationWindow(ObjectSubclass<imp::ApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl ApplicationWindow {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create ApplicationWindow")
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let window = imp::ApplicationWindow::from_instance(self);

        let (width, height) = self.default_size();

        window.settings.set_int("window-width", width)?;
        window.settings.set_int("window-height", height)?;

        window
            .settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let window = imp::ApplicationWindow::from_instance(self);

        let width = window.settings.int("window-width");
        let height = window.settings.int("window-height");
        let is_maximized = window.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    pub fn update(&self, model: &ApplicationModel, changes: &Changes) {
        debug!("GtkApplicationWindow<Application>::update {:?}", changes);
        let window = imp::ApplicationWindow::from_instance(self);
        let document = model.document();
        let modified = document.modified();
        window.modified.set_visible(modified);
        if changes.text {
            window.bodytext.buffer().set_text(document.text().as_str());
            debug!("GtkApplicationWindow<Application>::update m {}", modified);
        }
        if changes.filename {
            match document.filename() {
                Some(title) => window.title.set_text(title.as_str()),
                None => window.title.set_text(""),
            }
        }
        if changes.status_message {
            let text = match model.status_message() {
                StatusMessage::None => String::new(),
                StatusMessage::SavingFile => gettext("Saving file..."),
                StatusMessage::OpeningFile => gettext("Opening file..."),
                StatusMessage::FileSaveFinished(Ok(())) => format!(
                    "{}: \"{}\"",
                    gettext("File saved to"),
                    Self::filepath_string(model)
                ),
                StatusMessage::FileOpenFinished(Ok(())) => String::new(),
                StatusMessage::FileSaveFinished(Err(_)) => format!(
                    "{}: \"{}\"!",
                    gettext("Could not save file"),
                    Self::filepath_string(model)
                ),
                StatusMessage::FileOpenFinished(Err(_)) => format!(
                    "{}: \"{}\"!",
                    gettext("Could not open file"),
                    Self::filepath_string(model)
                ),
            };
            window.status_bar.set_text(text.as_str());
        }
    }

    fn filepath_string(model: &ApplicationModel) -> String {
        match model.document().filepath() {
            Some(path) => match path.into_os_string().into_string() {
                Ok(s) => s,
                Err(_) => model.document().filename().unwrap_or_else(String::new),
            },
            None => String::new(),
        }
    }

    fn get_buffer_value(buffer: gtk::TextBuffer) -> String {
        let start = buffer.start_iter();
        let end = buffer.end_iter();
        buffer.text(&start, &end, true).to_string()
    }

    pub fn transmit(&self, tx: Sender<Action>) {
        let window = imp::ApplicationWindow::from_instance(self);
        let buffer = window.bodytext.buffer();
        let tx_local = tx.clone();
        buffer
            .connect("insert-text", true, move |args| {
                let buffer: gtk::TextBuffer = args[0].get().unwrap();
                let value = Self::get_buffer_value(buffer);
                debug!(
                    "GtkApplicationWindow<Application>::transmit insert-text {}",
                    value
                );
                tx_local.send(DocumentChanged(value)).ok();
                None
            })
            .ok();
        let tx_local = tx;
        buffer
            .connect("delete-range", true, move |args| {
                let buffer: gtk::TextBuffer = args[0].get().unwrap();
                let value = Self::get_buffer_value(buffer);
                debug!(
                    "GtkApplicationWindow<Application>::transmit delete-range {}",
                    value
                );
                tx_local.send(DocumentChanged(value)).ok();
                None
            })
            .ok();
    }

    pub fn undo(&self) {
        let window = imp::ApplicationWindow::from_instance(self);
        window.bodytext.buffer().undo();
    }

    pub fn redo(&self) {
        let window = imp::ApplicationWindow::from_instance(self);
        window.bodytext.buffer().redo();
    }

    pub fn can_undo(&self) -> bool {
        let window = imp::ApplicationWindow::from_instance(self);
        window.bodytext.buffer().can_undo()
    }

    pub fn can_redo(&self) -> bool {
        let window = imp::ApplicationWindow::from_instance(self);
        window.bodytext.buffer().can_redo()
    }
}
