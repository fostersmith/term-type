use std::io;

use std::time::{Instant, Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;

mod app;
mod ui;
use crate::app::App;
use crate::ui::draw;



fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
	let mut app = App::default();
	let refresh_wait = Duration::from_millis(250);
	let result = run(&mut app, &mut terminal, refresh_wait);
	ratatui::restore();
	result
}

fn run(app: &mut App, terminal: &mut DefaultTerminal, refresh_wait: Duration) -> io::Result<()> {
	let mut last_tick = Instant::now();
	while !app.quit {
		terminal.draw(|frame| draw(frame, app))?;
		handle_events(app, refresh_wait, last_tick)?;
		last_tick = Instant::now();
	}
	Ok(())
}

fn handle_events(app: &mut App, refresh_wait: Duration, last_tick: Instant) -> io::Result<()> {
	let timeout = refresh_wait.saturating_sub(last_tick.elapsed());
	if event::poll(timeout)? {
		match event::read()? {
			Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
				handle_key_event(key_event, app)
			}
			_ => {}
		}
	}
	Ok(())
}

fn handle_key_event(key_event: KeyEvent, app: &mut App) {
	let mut c_opt = key_event.code.as_char();
	if let Some(c) = c_opt.take() {
		if c == ' ' {
			app.on_space();
		} else {
			app.on_key(c);
		}
	} else {
		match key_event.code {
			KeyCode::Backspace | KeyCode::Delete => app.on_del(),
			KeyCode::Enter => app.on_enter(),
			KeyCode::Esc => app.on_esc(),
			_ => {},	
		}
	}
}
