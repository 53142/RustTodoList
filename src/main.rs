mod app;
mod models;
mod storage;
mod ui;

use color_eyre::Result;
use models::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let app = App::new()?;

    let result = app.run(&mut terminal);

    ratatui::restore();

    result
}