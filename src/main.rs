use std::io::Result;
use std::time::Duration;
use std::time::Instant;

use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::Frame;
use tui::Tui;

mod gemini;
mod tui;

struct App {
    should_exit: bool,
    show_input: bool,
    input_field: InputField,
    response_window: ResponseWindow,
}

struct ResponseWindow {
    response: Vec<String>,
}

impl ResponseWindow {
    fn default() -> Self {
        Self { response: vec![] }
    }
}

impl Widget for &ResponseWindow {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let window = Block::new().title("Gemini").borders(Borders::ALL);

        window.render(area, buf);
    }
}

struct InputField {
    input: String,
}

impl InputField {
    fn default() -> Self {
        Self {
            input: String::from(""),
        }
    }
}

impl Widget for &InputField {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let input_field = Block::new()
            .title("Ask something")
            .borders(Borders::ALL)
            .border_style(Style::new().bold());

        input_field.render(area, buf);
    }
}

impl App {
    fn run(&mut self, terminal: &mut Tui) -> Result<()> {
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
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Max(8)])
            .split(frame.area());

        self.response_window.render(layout[0], frame.buffer_mut());
        self.input_field.render(layout[1], frame.buffer_mut());
    }

    fn handle_events(&mut self, last_tick: Instant) -> Result<()> {
        let timeout = Duration::from_millis(50)
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        self.exit();
                    }
                    KeyCode::Enter => {
                        self.submit_input(&self.input_field.input);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn submit_input(&self, input: &str) -> Result<()> {
        Ok(())
    }

    fn exit(&mut self) {
        self.should_exit = true;
    }
}

fn main() -> Result<()> {
    println!("Hello, world!");

    let mut app = App {
        should_exit: false,
        show_input: true,
        input_field: InputField::default(),
        response_window: ResponseWindow::default(),
    };

    // Create backend
    let mut tui = tui::init()?;

    let app_result = app.run(&mut tui);

    tui::restore()?;

    app_result
}
