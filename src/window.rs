use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use log::debug;

use super::actions::Action;
use super::actions::Action::DocumentChanged;
use crate::glib::Sender;

use super::application_model::ApplicationModel;
use crate::application::Application;
use crate::config::{APP_ID, PROFILE};

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/bernardigiri/TextEdit2/ui/window.ui")]
    pub struct ApplicationWindow {
        #[template_child]
        pub headerbar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub bodytext: TemplateChild<gtk::TextView>,
        pub settings: gio::Settings,
        #[template_child]
        pub save_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub open_button: TemplateChild<gtk::Button>,
    }

    impl Default for ApplicationWindow {
        fn default() -> Self {
            Self {
                headerbar: TemplateChild::default(),
                bodytext: TemplateChild::default(),
                save_button: TemplateChild::default(),
                open_button: TemplateChild::default(),
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

    pub fn update(&self, model: &ApplicationModel) {
        debug!("GtkApplicationWindow<Application>::update");
        let window = imp::ApplicationWindow::from_instance(self);
        window
            .bodytext
            .buffer()
            .set_text(model.document().text().as_str());
    }

    pub fn transmit(&self, tx: Sender<Action>) {
        let window = imp::ApplicationWindow::from_instance(self);
        window.bodytext.buffer().connect_changed(move |buffer| {
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let value = buffer.text(&start, &end, true).to_string();
            tx.send(DocumentChanged(value)).ok();
        });
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
