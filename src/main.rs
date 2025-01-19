mod app;
mod config;
mod file_handler;

mod event_handler;
mod helper;
mod layout;
mod screens;
mod storage;

use app::Tukai;
use config::TukaiConfigBuilder;
use event_handler::EventHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut terminal = ratatui::init();
  let mut event_handler = EventHandler::new();

  let app_config = TukaiConfigBuilder::new().build();

  terminal.clear()?;

  let app_result = Tukai::try_new(&mut event_handler, app_config)?
    .run(&mut terminal)
    .await;

  ratatui::restore();

  app_result
}
