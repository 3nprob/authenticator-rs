mod exporting;
mod helpers;
mod main_window;
mod model;
mod ui;

// extern crate gio;
// extern crate glib;
extern crate gtk;
// extern crate gdk;

use gtk::gdk::Display;
use gtk::{
    Application, ApplicationWindow, Box as Box_, Button, ComboBoxText, CssProvider, Entry, Orientation, StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION,
};

// use std::sync::{Arc, Mutex};
//
use gettextrs::*;
use gtk::prelude::*;
use log::info;
use rusqlite::Connection;
//
// use main_window::MainWindow;
//
// use crate::helpers::runner;
// use crate::helpers::ConfigManager;

// mod main_window;
//
// mod exporting;
// mod helpers;
// mod model;
// mod ui;

use crate::helpers::{runner, ConfigManager};
use crate::main_window::MainWindow;
use log4rs::config::Config;
use log4rs::file::{Deserializers, RawConfig};
use std::env::args;
use std::sync::{Arc, Mutex};

const NAMESPACE: &str = "uk.co.grumlimited.authenticator-rs";
const NAMESPACE_PREFIX: &str = "/uk/co/grumlimited/authenticator-rs";

const GETTEXT_PACKAGE: &str = "authenticator-rs";
const LOCALEDIR: &str = "/usr/share/locale";

fn main() {
    let resource = {
        match gtk::gio::Resource::load(format!("data/{}.gresource", NAMESPACE)) {
            Ok(resource) => resource,
            Err(_) => gtk::gio::Resource::load(format!("/usr/share/{}/{}.gresource", NAMESPACE, NAMESPACE)).unwrap(),
        }
    };

    gtk::gio::functions::resources_register(&resource);

    let application = gtk::Application::new(Some(NAMESPACE), Default::default()).expect("Initialization failed...");

    application.connect_startup(move |_| {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource(format!("{}/{}", NAMESPACE_PREFIX, "style.css").as_str());

        StyleContext::add_provider_for_display(
            &Display::get_default().expect("Error initializing gtk css provider."),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Prepare i18n
        setlocale(LocaleCategory::LcAll, "");
        bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR);
        textdomain(GETTEXT_PACKAGE);

        configure_logging();

        match ConfigManager::check_configuration_dir() {
            Ok(()) => info!("Reading configuration from {}", ConfigManager::path().display()),
            Err(e) => panic!(e),
        }
    });

    application.connect_activate(move |app| {
        let mut gui = MainWindow::new();

        let mut connection = ConfigManager::create_connection().unwrap();

        // SQL migrations
        runner::run(&mut connection).unwrap();

        let connection: Arc<Mutex<Connection>> = Arc::new(Mutex::new(connection));

        gui.set_application(&app, connection);

        info!("Authenticator RS initialised");
    });

    application.run(&[]);
}

/**
* Loads log4rs yaml config from gResource.
* And in the most convoluted possible way, feeds it to Log4rs.
*/
fn configure_logging() {
    let log4rs_yaml =
        gtk::gio::functions::resources_lookup_data(format!("{}/{}", NAMESPACE_PREFIX, "log4rs.yaml").as_str(), gtk::gio::ResourceLookupFlags::NONE).unwrap();
    let log4rs_yaml = log4rs_yaml.to_vec();
    let log4rs_yaml = String::from_utf8(log4rs_yaml).unwrap();

    // log4rs-0.12.0/src/file.rs#592
    let config = serde_yaml::from_str::<RawConfig>(log4rs_yaml.as_str()).unwrap();
    let (appenders, _) = config.appenders_lossy(&Deserializers::default());

    // log4rs-0.12.0/src/priv_file.rs#deserialize(config: &RawConfig, deserializers: &Deserializers)#186
    let config = Config::builder().appenders(appenders).loggers(config.loggers()).build(config.root()).unwrap();

    log4rs::init_config(config).unwrap();
}
