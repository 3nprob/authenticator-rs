use crate::helpers::ConfigManager;
use crate::main_window::MainWindow;
use crate::NAMESPACE_PREFIX;
use gettextrs::*;
use gtk::cairo::glib::SignalHandlerId;
use gtk::glib::{Receiver, Sender};
use gtk::prelude::*;
use gtk::{Button, PopoverMenu};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub trait Exporting {
    fn export_accounts(&self, popover: gtk::PopoverMenu, connection: Arc<Mutex<Connection>>) -> Box<dyn Fn(&gtk::Button)>;

    fn import_accounts(&self, popover: gtk::PopoverMenu, connection: Arc<Mutex<Connection>>) -> Box<dyn Fn(&gtk::Button)>;

    fn popup_close(popup: gtk::Window) -> Box<dyn Fn(&[gtk::glib::Value]) -> Option<gtk::glib::Value>>;
}

impl Exporting for MainWindow {
    fn export_accounts(&self, popover: PopoverMenu, connection: Arc<Mutex<Connection>>) -> Box<dyn Fn(&Button)> {
        let gui = self.clone();
        Box::new(move |_b: &gtk::Button| {
            popover.set_visible(false);

            let builder = gtk::Builder::from_resource(format!("{}/{}", NAMESPACE_PREFIX, "error_popup.ui").as_str());

            let dialog: gtk::FileChooserDialog = builder.get_object("dialog").unwrap();

            let export_account_error: gtk::Window = builder.get_object("error_popup").unwrap();
            let export_account_error_body: gtk::Label = builder.get_object("error_popup_body").unwrap();

            export_account_error_body.set_label(&gettext("Could not export accounts!"));

            {
                let export_account_error = export_account_error.clone();
                builder.connect_local("export_account_error_close", true, move |e| {
                    Self::popup_close(export_account_error.clone());
                    None
                });
            }

            dialog.show();
            let gui = gui.clone();
            let connection = connection.clone();
            dialog.connect_response(move |w, response| {
                match response {
                    gtk::ResponseType::Accept => {
                        let path = w.get_file().unwrap();
                        let path = PathBuf::from(path.to_string());

                        let (tx, rx): (Sender<bool>, Receiver<bool>) = gtk::glib::MainContext::channel::<bool>(gtk::glib::PRIORITY_DEFAULT);

                        // sensitivity is restored in refresh_accounts()
                        // gui.accounts_window.accounts_container.set_sensitive(false);
                        gui.pool.spawn_ok(ConfigManager::save_accounts(path, connection.clone(), tx));

                        let export_account_error = export_account_error.clone();
                        rx.attach(None, move |success| {
                            if !success {
                                export_account_error.set_title(Some(&gettext("Error")));
                                export_account_error.show();
                            }

                            // gui.accounts_window.refresh_accounts(&gui, connection.clone());

                            gtk::glib::Continue(true)
                        });

                        w.close();
                    }
                    _ => w.close(),
                }
            });
        })
    }

    fn import_accounts(&self, popover: gtk::PopoverMenu, connection: Arc<Mutex<Connection>>) -> Box<dyn Fn(&gtk::Button)> {
        let gui = self.clone();
        Box::new(move |_b: &gtk::Button| {
            popover.set_visible(false);

            let builder = gtk::Builder::from_resource(format!("{}/{}", NAMESPACE_PREFIX, "error_popup.ui").as_str());

            let dialog: gtk::FileChooserDialog = builder.get_object("dialog").unwrap();

            let export_account_error: gtk::Window = builder.get_object("error_popup").unwrap();
            export_account_error.set_title(Some(&gettext("Error")));

            let export_account_error_body: gtk::Label = builder.get_object("error_popup_body").unwrap();

            export_account_error_body.set_label(&gettext("Could not import accounts!"));

            {
                let export_account_error = export_account_error.clone();
                builder.connect_local("export_account_error_close", true, move |e| {
                    Self::popup_close(export_account_error.clone());
                    None
                });
            }

            dialog.show();
            let gui = gui.clone();
            let connection = connection.clone();
            dialog.connect_response(move |w, response| {
                match response {
                    gtk::ResponseType::Accept => {
                        let path = w.get_file().unwrap();
                        let path = PathBuf::from(path.to_string());

                        let (tx, rx): (Sender<bool>, Receiver<bool>) = gtk::glib::MainContext::channel::<bool>(gtk::glib::PRIORITY_DEFAULT);

                        // sensitivity is restored in refresh_accounts()
                        // gui.accounts_window.accounts_container.set_sensitive(false);
                        gui.pool.spawn_ok(ConfigManager::restore_account_and_signal_back(path, connection.clone(), tx));

                        let export_account_error = export_account_error.clone();
                        rx.attach(None, move |success| {
                            if !success {
                                export_account_error.show();
                            }

                            // gui.accounts_window.refresh_accounts(&gui, connection.clone());

                            gtk::glib::Continue(true)
                        });
                    }
                    _ => w.close(),
                }
            });
        })
    }

    fn popup_close(popup: gtk::Window) -> Box<dyn Fn(&[gtk::glib::Value]) -> Option<gtk::glib::Value>> {
        Box::new(move |_param: &[gtk::glib::Value]| {
            popup.hide();
            None
        })
    }
}
