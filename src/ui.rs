use std::str::Chars;

use ratatui::{
	layout::{Constraint, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Span, Line},
	widgets::{Wrap, Block, Paragraph},
	Frame,
};

use crate::app::App;
use crate::app::AppState;

pub fn draw(frame: &mut Frame, app: &mut App) {
	let chunks = Layout::vertical(
		[Constraint::Length(3),Constraint::Min(0)])
		.split(frame.area());

	let title = Line::from(vec![
	   	" TermType v".into(),
	   	env!("CARGO_PKG_VERSION").into(),
	   	" - Written by Foster Smith ".into(),
	]);
	
	let title_paragraph = Paragraph::new(title.centered())
		.block(Block::bordered());
	frame.render_widget(title_paragraph, chunks[0]);
	
	match app.state {
		AppState::Typing => draw_typing(frame, app, chunks[1]),
		AppState::Menu => draw_menu(frame, chunks[1]),
		AppState::Stats => draw_stats(frame, app, chunks[1]),
	}
}

fn draw_menu(frame: &mut Frame, area: Rect) {
	let menu_paragraph = Paragraph::new(
			Line::from("Press Enter to Start Test").centered()
		)
		.block(Block::bordered());
	frame.render_widget(menu_paragraph, area);
}

fn draw_stats(frame: &mut Frame, app: &mut App, area: Rect) {
	let stats = &app.active_stats;
	let stats_paragraph = Paragraph::new(vec![
		Line::from(format!("wpm: {}", stats.wpm)),
		Line::from(format!("wpm raw: {}", stats.wpm_raw)),
		Line::from(format!("acc: {}%", stats.acc*(100 as f32))),
		Line::from(format!("words: {}/{}", stats.word_corr, stats.word_total)),
		Line::from(format!("chars: {}/{}", stats.char_corr, stats.char_total)),
		Line::from(format!("test duration (s): {}", stats.duration_s)),
	])
	.block(Block::bordered());
	frame.render_widget(stats_paragraph, area);
}

fn draw_typing(frame: &mut Frame, app: &mut App, area: Rect) {
	let session = &mut app.active_session;	

	let mut input_spans: Vec<Span> = vec![];
	let target_words = &mut session.target_words;
	let words_to_render: usize;
	let input_len = session.input.len();

	match target_words.len() {
		Some(len) => words_to_render = len,
		None => words_to_render = input_len + 10,
	}

	for i in 0..words_to_render {
		let word = target_words
			.get_word_at(i)
			.expect("Ran out of words unexpectedly! 
			Check WordGenerator implementation");
		let mut typed_chars_opt: Option<Chars> = None;
		if i < session.input.len() {
			typed_chars_opt = Option::from(
				session.input[i].chars()
			);
		}

		// Chars in target
		for ch in word.chars() {
			let mut typed_char_opt: Option<char> = None;
			if let Some(typed_chars) = typed_chars_opt.as_mut() {
				typed_char_opt = typed_chars.next();			
			}

			let style: Style;
			if typed_char_opt.is_none() {
				style = Style::default();
			} else if typed_char_opt.expect("") == ch {
				style = Style::default().fg(Color::Green)
					.add_modifier(Modifier::BOLD);
			} else {
				style = Style::default().fg(Color::Red)
					.add_modifier(Modifier::BOLD)
					.add_modifier(Modifier::CROSSED_OUT);
			}
			let s: Span = Span::styled(ch.to_string(), style);
			input_spans.push(s);
		}
		// Overtyped chars
		if let Some(typed_chars) = typed_chars_opt.as_mut() {
			let overtyped_str = typed_chars.as_str();
			let overtyped_span = Span::styled(overtyped_str,
				Style::default().fg(Color::Red)
					.add_modifier(Modifier::ITALIC)
			);
			input_spans.push(overtyped_span);
		}
		input_spans.push(Span::from(" "));
	}

	let mut age_opt = session.get_age_s();
	let mut bottom_title_string = "".to_string();
	
	if let Some(age_f64) = age_opt.take() {
		bottom_title_string = (age_f64 as i64).to_string();
	}
	
	let temp_line = Line::from(input_spans);
	let block = Block::bordered()
		.title_bottom(bottom_title_string);

	let typing_paragraph = 
		Paragraph::new(temp_line.centered())
		.block(block)
		.wrap(Wrap { trim: true });

	frame.render_widget(typing_paragraph, area);
}

