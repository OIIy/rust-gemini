use std::time::Duration;
use std::time::Instant;

use gemini::Gemini;
use gemini::GeminiError;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEvent;
use ratatui::crossterm::event::KeyModifiers;
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
use tokio::runtime::Runtime;
use tokio::task;
use tui::Tui;

mod error;
mod gemini;
mod tui;

pub use self::error::Error;
pub use self::error::Result;

struct App {
    should_exit: bool,
    show_input: bool,
    input_field: InputField,
    response_window: ResponseWindow,
    gemini_client: Gemini,
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

        let mut response = String::from("Nothing yet.");

        if !self.response.is_empty() {
            response = self.response[0].clone();
        }

        let response_text = Paragraph::new(response).block(window);

        response_text.render(area, buf);
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

    fn clear(&mut self) {
        self.input = String::from("");
    }
}

impl Widget for &InputField {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let input_field = Block::new()
            .title("Ask something")
            .borders(Borders::ALL)
            .border_style(Style::new().bold());

        let input_text = Paragraph::new(self.input.clone()).block(input_field);

        input_text.render(area, buf);
    }
}

impl App {
    async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        let last_tick = Instant::now();

        loop {
            if self.should_exit {
                return Ok(());
            }

            terminal.draw(|f| self.render_ui(f))?;

            self.handle_events(last_tick).await;
        }
    }

    fn render_ui(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(frame.area());

        self.response_window.render(layout[0], frame.buffer_mut());
        self.input_field.render(layout[1], frame.buffer_mut());
    }

    // TODO: Refactor using non-blocking event reads and an event stream
    async fn handle_events(&mut self, last_tick: Instant) -> Result<()> {
        let timeout = Duration::from_millis(50)
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                if modifiers == KeyModifiers::CONTROL {
                    if let KeyCode::Char('q') = code {
                        self.exit();
                    }
                } else {
                    match code {
                        KeyCode::Enter => {
                            self.submit_input().await;
                        }
                        KeyCode::Backspace => {
                            self.input_field.input.pop();
                        }
                        _ => self.input_field.input += &code.to_string(),
                    }
                }
            }
        }

        Ok(())
    }

    async fn submit_input(&mut self) -> Result<()> {
        let response = self
            .gemini_client
            .ask(self.input_field.input.as_ref())
            .await;

        if response.is_ok() {
            self.response_window.response = response.unwrap();
        }

        self.input_field.clear();

        Ok(())
    }

    fn exit(&mut self) {
        self.should_exit = true;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Here's what we have to do:
    // 1. Get a response from Gemini to display.
    // 2. Use background tasks to handle events/async actions
    // 3. Proper Error Handling
    // 4. Tests
    // 5. Health checks
    // 6. Prettify & Refactor

    let api_key = "AIzaSyC0oK8pgMdT1zM0VouuWxlinJJs_brulkM";
    let gemini_model = "gemini-1.5-flash";
    let gemini = gemini::Gemini::new(Some(api_key), Some(gemini_model));

    let mut app = App {
        should_exit: false,
        show_input: true,
        input_field: InputField::default(),
        response_window: ResponseWindow::default(),
        gemini_client: gemini,
    };

    // Create backend
    let mut tui = tui::init()?;

    match app.run(&mut tui).await {
        Ok(_) => println!("Exit code 0"),
        Err(e) => println!("ERROR: {}", e),
    }

    tui::restore()?;

    Ok(())
}
