use gtk4::prelude::*;
use gtk4::Application;

mod desktop_file;
mod ui;

use ui::MainWindow;

const APP_ID: &str = "com.ubuntu.DesktopManager";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = MainWindow::new(app);
    window.present();
}
