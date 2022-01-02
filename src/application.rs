use gettextrs::gettext;
use log::{debug, info};

use glib::{clone, Continue, MainContext, PRIORITY_DEFAULT};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};

use std::cell::RefCell;
use std::rc::Rc;

use super::actions::Action;
use super::actions::Action::*;
use super::application_model::ApplicationModel;
use super::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use super::window::ApplicationWindow;
use crate::glib::Sender;

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::OnceCell;

    #[derive(Debug)]
    pub struct Application {
        pub window: OnceCell<WeakRef<ApplicationWindow>>,
        pub model: Rc<RefCell<ApplicationModel>>,
        pub undo_action: gio::SimpleAction,
        pub redo_action: gio::SimpleAction,
    }

    impl Default for Application {
        fn default() -> Self {
            let undo_action = gio::SimpleAction::new("undo", None);
            let redo_action = gio::SimpleAction::new("redo", None);
            Self {
                window: OnceCell::default(),
                model: Rc::default(),
                undo_action,
                redo_action,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self, app: &Self::Type) {
            debug!("GtkApplication<Application>::activate");

            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.show();
                window.present();
                return;
            }

            let window = ApplicationWindow::new(app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT);

            let model_rc = app.model();
            {
                let local_m = model_rc.clone();
                let mut model = local_m.borrow_mut();
                model.transmit(tx.clone());
            }
            app.transmit(tx);

            app.main_window().present();
            let local_app = app.clone();

            rx.attach(None, move |action| {
                let update_view = {
                    let mut model = model_rc.borrow_mut();
                    model.update(action)
                };
                if update_view {
                    local_app.update();
                }
                Continue(true)
            });
        }

        fn startup(&self, app: &Self::Type) {
            debug!("GtkApplication<Application>::startup");
            self.parent_startup(app);

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            app.setup_css();
            app.setup_gactions();
            app.setup_accels();
        }
    }

    impl GtkApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Default for Application {
    fn default() -> Self {
        Application::new()
    }
}

impl Application {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &Some(APP_ID)),
            ("flags", &gio::ApplicationFlags::empty()),
            ("resource-base-path", &Some("/com/bernardigiri/TextEdit2/")),
        ])
        .expect("Application initialization failed...")
    }

    fn transmit(&self, tx: Sender<Action>) {
        let window = self.main_window();
        window.transmit(tx);
    }

    fn update(&self) {
        debug!("GtkApplication<Application>::update");
        let model_ref = self.model();
        let model = model_ref.borrow();
        let window = self.main_window();
        window.update(&model);
        let imp = imp::Application::from_instance(self);
        imp.undo_action.set_enabled(window.can_undo());
        imp.redo_action.set_enabled(window.can_redo());
    }

    fn model(&self) -> Rc<RefCell<ApplicationModel>> {
        let imp = imp::Application::from_instance(self);
        imp.model.clone()
    }

    fn main_window(&self) -> ApplicationWindow {
        let imp = imp::Application::from_instance(self);
        imp.window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let action = gio::SimpleAction::new("quit", None);
        action.connect_activate(clone!(@weak self as app => move |_, _| {
            // This is needed to trigger the delete event and saving the window state
            app.main_window().close();
            app.quit();
        }));
        self.add_action(&action);

        // About
        let action = gio::SimpleAction::new("about", None);
        action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_about_dialog();
        }));
        self.add_action(&action);

        // Save
        let action = gio::SimpleAction::new("save", None);
        action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.save_file();
        }));
        self.add_action(&action);

        // Save As
        let action = gio::SimpleAction::new("save-as", None);
        action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.save_file_as();
        }));
        self.add_action(&action);

        // Open
        let action = gio::SimpleAction::new("open", None);
        action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.open_file();
        }));
        self.add_action(&action);

        // New
        let action = gio::SimpleAction::new("new", None);
        action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.new_file();
        }));
        self.add_action(&action);

        // Toggle actions
        {
            let imp = imp::Application::from_instance(self);
            // Undo
            let action = &imp.undo_action;
            action.connect_activate(clone!(@weak self as app => move |_, _| {
                app.undo();
            }));
            self.add_action(action);

            // Redo
            let action = &imp.redo_action;
            action.connect_activate(clone!(@weak self as app => move |_, _| {
                app.redo();
            }));
            self.add_action(action);
        }
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.new", &["<primary>n"]);
        self.set_accels_for_action("app.open", &["<primary>o"]);
        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("app.redo", &["<primary><shift>z"]);
        self.set_accels_for_action("app.save", &["<primary>s"]);
        self.set_accels_for_action("app.undo", &["<primary>z"]);
    }

    fn setup_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/com/bernardigiri/TextEdit2/style.css");
        if let Some(display) = gdk::Display::default() {
            gtk::StyleContext::add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn show_about_dialog(&self) {
        let logo_file = gio::File::for_path("/com/bernardigiri/TextEdit2/ui/logo.svg");
        let logo = gtk::IconPaintableBuilder::new().file(&logo_file).build();
        let dialog = gtk::AboutDialogBuilder::new()
            .program_name("TextEdit 2")
            .logo(&logo)
            .logo_icon_name(APP_ID)
            .license_type(gtk::License::MitX11)
            .website("https://github.com/BernardIgiri/TextEdit2")
            .version(VERSION)
            .transient_for(&self.main_window())
            .translator_credits(&gettext("translator-credits"))
            .modal(true)
            .authors(vec!["Bernard Igiri".into()])
            .artists(vec!["Bernard Igiri".into()])
            .build();

        dialog.show();
    }

    fn save_file(&self) {
        debug!("GtkApplication<Application>::save_file");
        let model_rc = self.model();
        let model = model_rc.borrow_mut();
        match model.document().filepath() {
            None => {
                self.save_file_as();
            }
            Some(path) => {
                model.send(SaveFile(path));
            }
        }
    }

    fn save_file_as(&self) {
        debug!("GtkApplication<Application>::save_file_as");
        let file_chooser = gtk::FileChooserDialog::new(
            Some(&gettext("Save As")),
            Some(&self.main_window()),
            gtk::FileChooserAction::Save,
            &[
                (&gettext("Save"), gtk::ResponseType::Ok),
                (&gettext("Cancel"), gtk::ResponseType::Cancel),
            ],
        );

        let model_rc = self.model();

        file_chooser.connect_response(
            move |d: &gtk::FileChooserDialog, response: gtk::ResponseType| {
                if response == gtk::ResponseType::Ok {
                    debug!("GtkApplication<Application>::open_file Ok");
                    let file = d.file().expect("Couldn't get file");
                    let model = model_rc.borrow();
                    model.send(SaveFile(file.path().unwrap()));
                }
                d.close();
            },
        );

        file_chooser.show();
    }

    fn open_file(&self) {
        debug!("GtkApplication<Application>::open_file");
        let file_chooser = gtk::FileChooserDialog::new(
            Some(&gettext("Open File")),
            Some(&self.main_window()),
            gtk::FileChooserAction::Open,
            &[
                (&gettext("Open"), gtk::ResponseType::Ok),
                (&gettext("Cancel"), gtk::ResponseType::Cancel),
            ],
        );

        let model_rc = self.model();

        file_chooser.connect_response(
            move |d: &gtk::FileChooserDialog, response: gtk::ResponseType| {
                if response == gtk::ResponseType::Ok {
                    debug!("GtkApplication<Application>::open_file Ok");
                    let file = d.file().expect("Couldn't get file");
                    let model = model_rc.borrow();
                    model.send(OpenFile(file.path()));
                }
                d.close();
            },
        );

        file_chooser.show();
    }

    fn new_file(&self) {
        debug!("GtkApplication<Application>::new_file");
        let model_rc = self.model();
        let model = model_rc.borrow();
        model.send(OpenFile(None));
    }

    fn undo(&self) {
        debug!("GtkApplication<Application>::undo");
        self.main_window().undo();
    }

    fn redo(&self) {
        debug!("GtkApplication<Application>::redo");
        self.main_window().redo();
    }

    pub fn run(&self) {
        info!("TextEdit 2 ({})", APP_ID);
        info!("Version: {} ({})", VERSION, PROFILE);
        info!("Datadir: {}", PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}
