use tokio::sync::mpsc;
use tokio::time;
use crate::constants::colors;
use crate::helper::get_color_rgb;
use crate::tools::loader::Loader;
use crate::windows::{
  typing_window::{TypingWindow,Stats},
  stats_window::StatsWindow
};

use crate::traits::Window;

use std::{collections::HashMap, io::{self, Write}, time::Duration};
use ratatui::{
  crossterm::event::{self, KeyCode, KeyEventKind, Event, KeyEvent},
  layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
  style::{Color, Style, Styled, Stylize},
  text::{Line, Span, Text},
  widgets::{block::{Position, Title}, Block, Borders, Clear, Paragraph},
  DefaultTerminal,
  Frame
};

#[derive(PartialEq, Hash, Eq)]
enum ActiveWindowEnum {
  Typing,
  Stats
}

pub struct App<'a> {
  is_exit: bool,
  has_done: bool,

  loader: Loader<'a>,
  active_window: ActiveWindowEnum,
  instructions: HashMap<ActiveWindowEnum, Vec<Span<'a>>>,
  typing_window: TypingWindow,
  stats_window: StatsWindow
}

impl<'a> App<'a> {

  pub fn new() -> Self {
    let mut instructions = HashMap::new();

    let typing_window_instructions = vec![
      Span::styled("<ESC>Exit", Style::default().fg(get_color_rgb(colors::PRIMARY))),
      Span::styled(" <Left>Typing", Style::default().fg(get_color_rgb(colors::SECONDARY))),
      Span::styled(" <Right>Stats", Style::default().fg(get_color_rgb(colors::SECONDARY))),
    ];

    let stats_window_instructions = vec![
      Span::styled("<ESC>Exit", Style::default().fg(Color::Yellow)),
      Span::styled(" <Left>Typing", Style::default().fg(Color::Green)),
      Span::styled(" <Right>Stats", Style::default().fg(Color::Red)),
    ];

    instructions.insert(ActiveWindowEnum::Typing, typing_window_instructions);
    instructions.insert(ActiveWindowEnum::Stats, stats_window_instructions);

    Self {
      has_done: false,
      is_exit: false,
      loader: Loader::new(),
      active_window: ActiveWindowEnum::Typing,
      instructions,
      typing_window: TypingWindow::default(),
      stats_window: StatsWindow::default()
    }
  }

  pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
    while !self.is_exit {
      terminal.draw(|frame| self.draw(frame))?;

      if self.typing_window.generated_text.len() == self.typing_window.input.len() {
        self.has_done = true;
      }

      if self.handle_events()? {
        break;
      };
    }

    Ok(())
  }

  fn draw(&mut self, frame: &mut Frame) {
    let main_layout = Layout::default()
      .constraints(vec![
        Constraint::Min(0),
        Constraint::Length(1)
      ])
      .split(frame.area());

    match self.active_window {
      ActiveWindowEnum::Typing => self.typing_window.render(frame, main_layout[0]),
      ActiveWindowEnum::Stats => self.stats_window.render(frame, main_layout[0])
    }

    self.render_instructions(frame, main_layout[1]);

    if self.has_done {
      self.render_popup(frame, self.typing_window.stats.clone());
    }
  }

  fn handle_window_events(&mut self, key: KeyEvent) {
    match self.active_window {
      ActiveWindowEnum::Typing => self.typing_window.handle_events(key),
      // ActiveWindowEnum::Path => self.path_window.handle_events(key),
      // ActiveWindowEnum::Results => self.search.handle_events(key),
      _ => ()
    }
  }

  fn handle_events(&mut self) -> io::Result<bool> {
    if crossterm::event::poll(Duration::from_millis(100))? {
      if let event::Event::Key(key) = event::read()? {
        if key.code == KeyCode::Left {
          self.active_window = ActiveWindowEnum::Typing;
        } else if key.code == KeyCode::Right {
          self.active_window = ActiveWindowEnum::Stats;
        } else if key.code == KeyCode::Esc {
          self.is_exit = true;
          return Ok(true);
        }

        self.handle_window_events(key);
      }
    }

    Ok(false)
  }

  fn render_instructions(&self, frame: &mut Frame, area: Rect) {
    let default_vec= Vec::new();
    let instructions_spans = self.get_window_instructions().unwrap_or(&default_vec);

    let instructions = Paragraph::new(
      Text::from(Line::from(instructions_spans.clone()))
    ).alignment(Alignment::Center);
    
    frame.render_widget(instructions, area);
  }

  // fn render_path(&self, frame: &mut Frame, area: Rect) {
  //   let border_color = self.get_window_border_color(ActiveWindowEnum::Path);
  //
  //   let block = Block::new()
  //     .borders(Borders::ALL)
  //     .border_style(Style::default().fg(border_color))
  //     .title(Title::from("[2]Folder").alignment(Alignment::Center));
  //
  //   let p = Paragraph::new(self.path_window.input.clone())
  //     .block(block);
  //
  //   frame.render_widget(p, area);
  // }

  // fn render_settings(&self, frame: &mut Frame, area: Rect) {
  //   let border_color = self.get_window_border_color(ActiveWindowEnum::Settings);
  //
  //   let block = Block::new()
  //     .borders(Borders::ALL)
  //     .border_style(Style::default().fg(border_color))
  //     .title(Title::from("[3]Settings").alignment(Alignment::Center));
  //
  //   let p = Paragraph::new("Results")
  //     .block(block);
  //
  //   frame.render_widget(p, area);
  // }

  fn render_popup(&mut self, frame: &mut Frame, stats: Stats) {
    let area = frame.area();

    let block = Block::bordered()
      .border_style(Style::new().fg(Color::Red));

    let text = Text::from(vec![
      Line::from("Nice you make it:)"),
      Line::from(format!("Error makes {}", stats.errors_count)),
      //Line::from(self.loader.get_slash()),
    ]);

    let p = Paragraph::new(text)
      .block(block)
      .alignment(Alignment::Center)
      .centered()
      .bold();

    let vertical = Layout::vertical([Constraint::Percentage(20)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(20)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);

    frame.render_widget(Clear, area);
    frame.render_widget(p, area);
  }

  fn get_window_border_color(&self, current_window: ActiveWindowEnum) -> Color {
    if self.active_window == current_window {
      Color::from_u32(0x805CBF)
    } else {
      Color::from_u32(0x00999999)
    }
  }

  fn get_window_instructions(&self) -> Option<&Vec<Span<'a>>> {
    match self.active_window {
      ActiveWindowEnum::Typing => self.instructions.get(&ActiveWindowEnum::Typing),
      ActiveWindowEnum::Stats => self.instructions.get(&ActiveWindowEnum::Stats),
    }
  }

}

