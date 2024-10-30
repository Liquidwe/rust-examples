use std::io;
use ratatui::DefaultTerminal;

mod app;
mod ui;

use crate::app::App;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
