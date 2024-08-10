use std::{
    io::{self},
    time::{Duration, Instant},
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    widgets::Paragraph,
    Frame,
};
use tui::Tui;

mod gemini;
mod tui;

struct App {
    should_exit: bool,
}

impl App {
    fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        let last_tick = Instant::now();

        loop {
            if self.should_exit {
                return Ok(());
            }

            terminal.draw(|f| self.render_ui(f))?;

            self.handle_events(last_tick)?;
        }
    }

    fn render_ui(&mut self, frame: &mut Frame) {
        frame.render_widget(Paragraph::new(String::from("Hello world!")), frame.area());
    }

    fn handle_events(&mut self, last_tick: Instant) -> io::Result<()> {
        let timeout = Duration::from_millis(50)
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    self.exit();
                }
            }
        }

        Ok(())
    }

    fn exit(&mut self) {
        self.should_exit = true;
    }
}

fn main() -> io::Result<()> {
    println!("Hello, world!");

    let mut app = App { should_exit: false };

    // Create backend
    let mut tui = tui::init()?;

    let app_result = app.run(&mut tui);

    tui::restore()?;

    app_result
}
