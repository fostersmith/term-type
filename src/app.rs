use std::time::{Duration, Instant};
use std::fs;
use rand::prelude::*;

const WORD_LIST_FILE: &str="src/english-1k.txt";

pub trait WordGenerator {
	fn get_word_at(&mut self, i: usize) -> Option<String>;
	fn get_word_at_frozen(&self, i: usize) -> Option<String>;
	fn len(&self) -> Option<usize>;
}

//TODO timed mode is not implemented yet, so not setting size will break the game
struct RandomWordGenerator {
	words: Vec<String>,
	size: Option<usize>,
	word_list: Vec<String>,
}
impl RandomWordGenerator {
	pub fn with_size(s: usize) -> Self {
		let mut generator = Self{ 
			words: Vec::<String>::with_capacity(s), 
			size: Some(s),
			word_list: Self::load_word_list(WORD_LIST_FILE),
		};
		for _ in 0..s {
			generator.words.push(generator.get_random_word());
		}
		generator
	}

	fn load_word_list(filepath: &str) -> Vec<String> {
		let word_str = fs::read_to_string(filepath)
			.expect(
				format!("Could read word list file '{}'", filepath).as_str()
			);
		
		let word_list: Vec<String> = word_str.split('\n')
			.map(|s| s.to_string())
			.collect();
		
		word_list
	}

	fn get_random_word(&self) -> String {
		let mut rng = rand::rng();
		return self.word_list.choose(&mut rng)
			.expect("expected to return a random word")
			.to_string();
	}

	fn add_words(&mut self, n: usize) {
		self.words.reserve(n);
		for _ in 0..n {
			self.words.push(self.get_random_word());
		}
	}
}

impl WordGenerator for RandomWordGenerator {	
	fn get_word_at(&mut self, index: usize) -> Option<String> {
		if index >= self.words.len() {
			self.add_words(index - self.words.len()+1);
		}

		return Some(self.words[index].clone());
	}
	fn get_word_at_frozen(&self, index: usize) -> Option<String> {
		if index >= self.words.len() {return None;}
		else {return Some(self.words[index].clone())}
	}
	fn len(&self) -> Option<usize> {
		match self.size {
			Some(size) => return Some(size),
			None => return None,
		}
	}
}

struct StaticWordGenerator {
	words: Vec<String>,
}
impl StaticWordGenerator {
	pub fn from(s: String) -> Self {
		let words: Vec<String> = s.split(' ')
			.map(|s| s.to_string())
			.collect();
		
		Self {
			words: words,
		}
	}
}
impl WordGenerator for StaticWordGenerator {
	fn get_word_at_frozen(&self, index: usize) -> Option<String> {
		if index >= self.words.len() {return None;}
		else {return Some(self.words[index].clone())}
	}

	fn get_word_at(&mut self, index: usize) -> Option<String> {
		self.get_word_at_frozen(index)
	}
	
	fn len(&self) -> Option<usize> {
		return Some(self.words.len());
	}
}
#[derive(PartialEq, Debug)]
enum SessionState {
	Idle,
	Active,
	Finished,
}

pub struct Session {
	state: 				SessionState,
	start_time: 		Option<Instant>,
	duration:			Option<Duration>,
	pub target_words:	Box<dyn WordGenerator>,
	pub target_text:	Vec<String>,
	pub input: 			Vec<String>,
}

impl Session {
	pub fn default() -> Self {
		//Self::from("The quick brown fox jumps over the lazy dog".to_string())
		Self::random_with_size(25)
	}

	pub fn random_with_size(s: usize) -> Self {
		Self {
			state: SessionState::Idle,
			start_time: None,
			duration: None,
			target_words: Box::from(RandomWordGenerator::with_size(s)),
			target_text: vec![],
			input: vec!["".to_string()],
		}
	}

	pub fn from(s: String) -> Self {
		let target_text: Vec<String> = s.split(' ')
			.map(|s| s.to_string())
			.collect();
		
		Self {
			state: 			SessionState::Idle,
			start_time:		None,
			duration:		None,
			target_words:	Box::from(StaticWordGenerator::from(s)),
			target_text: 	target_text,
			input:			vec!["".to_string()],
		}
	}

	pub fn start_session(&mut self) {
		assert_eq!(self.state, SessionState::Idle, 
				"Can't start active or ended session!");

		self.state = SessionState::Active;
		self.start_time = Some(Instant::now());
	}

	pub fn stop_session(&mut self){
		assert_eq!(self.state, SessionState::Active, 
				"Session ended before starting!");
		
		let start = self.start_time.expect("Start time was never set!");

		self.duration = Some(start.elapsed());
		self.state = SessionState::Finished;
	}

	pub fn on_char(&mut self, c: char) {
		if self.state == SessionState::Idle {
			self.start_session();
		}

		assert_eq!(self.state, SessionState::Active, 
				"Input received before session started!");
		
		let input_len = self.input.len();
		let last_word = self.input.last_mut()
				.expect("No words in input!");

		last_word.push(c);

		// check to end the session
		if self.target_words.len() == Some(input_len) {
			let last_target_word = self.target_words.get_word_at(input_len-1)
									.expect("empty target text!");
			
			if last_target_word == *last_word {
				self.stop_session();
			}
		}
	}

	pub fn on_space(&mut self) {
		if self.state == SessionState::Idle {
			return;
		}

		assert_eq!(self.state, SessionState::Active, 
				"Input received before session started!");
		
		let input_len = self.input.len();
		let last_word = self.input.last()
				.expect("No words in input!");
		
		if self.target_text.len() == input_len {
			self.stop_session();
			return;		
		}

		// ignore spaces if the last word is already empty
		if last_word != "" {
			self.input.push("".to_string());
		}
	}

	pub fn on_del(&mut self) {
		assert_eq!(self.state, SessionState::Active, 
				"Input received before session started!");
		
		let last_word = self.input.last_mut()
				.expect("No words in input!");
		
		if last_word == "" {
			if self.input.len() != 1 {
				self.input.pop(); // remove last word
			}
		} else {
			last_word.pop(); // remove last char
		}
	}

	pub fn get_age_s(&self) -> Option<f64> {
		if self.state == SessionState::Idle { return None };

		match self.start_time {
			Some(start) => return Some(start.elapsed().as_secs_f64()),
			None		=> return None,
		}
	}

	pub fn get_final_duration_s(&self) -> Option<f64> {
		match self.duration {
			Some(dur) => return Some(dur.as_secs_f64()),
			None => return None,
		}
	}

	pub fn get_input_words(&self) -> Vec<String> {
		self.input.clone()
	}

	pub fn get_attempted_words(&self) -> Vec<String> {
		let l = self.input.len();
		let mut words = Vec::with_capacity(l);
		let target = &self.target_words;

		for i in 0..l{
			match target.get_word_at_frozen(i) {
				Some(w) => words.push(w),
				None => break,
			}
		}

		words
	}

	pub fn get_cursor_word(&self) -> usize {
		return self.input.len()-1;
	}

	pub fn get_cursor_char(&self) -> usize {
		let last_word = self.input.last().expect("no words in input!");
		return last_word.len();
	}
}

#[derive(Default)]
pub struct SessionStats {
	pub wpm:			f32,
	pub wpm_raw:		f32,
	pub acc:			f32,
	pub char_corr:		i32,
	pub char_total:		i32,
	pub word_corr:		i32,
	pub word_total:		i32,
	pub duration_s:		f64,
}

impl SessionStats {

	pub fn from(session: &Session) -> Self {
		assert_eq!(session.state, SessionState::Finished,
				"Calculating stats on a session before it is finished");		
		
		// Calculate char_total, char_corr, word_total, word_corr

		let mut char_total = 0 as i32;
		let mut char_corr = 0 as i32;
		
		let mut word_total = 0 as i32;
		let mut word_corr = 0 as i32;
		
		// used to calculate wpm
		let mut correct_word_char_count = 0;

		let input_words = session.get_input_words();
		let attempted_words = session.get_attempted_words();

		let mut i = 0;
		for in_word in input_words {
			let att_word = &attempted_words[i];
			let (corr, ttl, is_correct) = Self::word_compare(&in_word.as_str(), &att_word.as_str());
			char_corr += corr;
			char_total += ttl;
			if is_correct {
				word_corr += 1;
				correct_word_char_count += ttl;
			}
			word_total += 1;
			i += 1;
		}
		
		// account for spaces
		// TODO move this into main loop
		correct_word_char_count += word_corr-1; 
		char_corr += word_corr-1;
		char_total += word_corr-1;
		
		let duration_s = session.get_final_duration_s()
			.expect("Calculating stats on a session without duration");
		let duration_min: f64 = duration_s / (60 as f64);
		
		// let wpm = (word_corr as f32) / (duration_min as f32);
		// let wpm_raw = (word_total as f32) / (duration_min as f32);
		
		let wpm = (correct_word_char_count as f32) / (5.0 * duration_min as f32);
		let wpm_raw = (char_total as f32) / (5.0 * duration_min as f32);

		let acc = (char_corr as f32) / (char_total as f32);
		
		Self {
			wpm: wpm, wpm_raw: wpm_raw, acc: acc, char_corr: char_corr,
			char_total: char_total, word_corr: word_corr, word_total: word_total,
			duration_s: duration_s
		}		
	}

	// Returns: (correct chars, total chars, word correct)
	fn word_compare(inp: &str, targ: &str) -> (i32, i32, bool) {
		let mut char_corr = 0;
		
		let mut inp_chars = inp.chars();
		let mut targ_chars = targ.chars();

		loop {
			match (inp_chars.next(), targ_chars.next()) {
				(Some(inp_ch), Some(targ_ch)) => {
					if inp_ch == targ_ch {
						char_corr += 1;
					}
				},
				_ => break
			}
		}

		let ttl_chars: i32;
		if targ.len() > inp.len() {
			ttl_chars = targ.len() as i32;
		} else {
			ttl_chars = inp.len() as i32;
		}
	
		return(char_corr, ttl_chars, char_corr == ttl_chars)
	}
}

#[derive(PartialEq, Debug)]
pub enum AppState {
	Menu,
	Typing,
	Stats,
}

pub struct App {
	pub state:			AppState,
	pub quit:			bool,
	pub active_session:	Session,
	pub active_stats:	SessionStats,
	default_text:		Option<String>,
	default_word_count: Option<usize>,
}

impl App {
	pub fn default() -> Self {
		Self {
			state: AppState::Menu,
			active_session: Session::default(),
			active_stats: SessionStats::default(),
			quit: false,
			default_text: None,
			default_word_count: None,
		}
	}

	pub fn from_str(default_text: String) -> Self {
		Self {
			state: AppState::Menu,
			active_session: Session::default(),
			active_stats: SessionStats::default(),
			quit: false,
			default_text: Some(default_text),
			default_word_count: None,
		}
	}

	pub fn with_word_count(word_count: usize) -> Self {
		Self {
			state: AppState::Menu,
			active_session: Session::default(),
			active_stats: SessionStats::default(),
			quit: false,
			default_text: None,
			default_word_count: Some(word_count),
		}
	}

	pub fn on_esc(&mut self) {
		self.quit = true;
	}

	pub fn on_enter(&mut self) {
		match self.state {
			AppState::Menu => self.open_typing(),
			AppState::Stats => self.open_menu(),
			_ => {}, // do nothing in typing mode	
		}
	}

	pub fn on_space(&mut self) {	
		match self.state {
			AppState::Typing => self.active_session.on_space(),
			_ => {}, // do nothing in menu or stats
		}
		self.check_state();
	}

	pub fn on_key(&mut self, c: char) {	
		match self.state {
			AppState::Typing => self.active_session.on_char(c),
			_ => {}, // do nothing in menu or stats (TODO)
		}
		self.check_state();
	}

	pub fn on_del(&mut self) {	
		match self.state {
			AppState::Typing => self.active_session.on_del(),
			_ => {}, // do nothing in menu or stats
		}
	}

	pub fn check_state(&mut self) {
		match self.state {
			AppState::Typing => {
				if self.active_session.state == SessionState::Finished {
					self.open_stats();
				}
			},
			_ => {}
		}
	}

	// helpers
	fn open_typing(&mut self) {
		if let Some(default_text) = &self.default_text {
			self.active_session = Session::from(default_text.clone());
		} else if let Some(default_count) = self.default_word_count {
			self.active_session = Session::random_with_size(default_count);
		} else {
			self.active_session = Session::default();
		}
		
		self.state = AppState::Typing;
	}
	fn open_stats(&mut self) {
		self.active_stats = SessionStats::from(&self.active_session);
		self.state = AppState::Stats;
	}
	fn open_menu(&mut self) {
		self.state = AppState::Menu;
	}
}

#[cfg(test)]
mod app_tests {
	use super::*;

	#[test]
	fn test_1() {
		let mut app = App::default();	
		app.on_enter();
		assert_eq!(app.state, AppState::Typing);
		app.on_key('a');
		app.on_key('b');
		app.on_key('c');
		assert_eq!(app.active_session.input[0], "abc");
		app.on_space();
		assert_eq!(app.active_session.input[1], "");
		app.on_key('d');
		app.on_key('e');
		app.on_key('f');
		assert_eq!(app.active_session.input[1], "def");
		
		assert_eq!(app.active_session.get_input_words(), 
			vec!["abc", "def"]
		);
		assert_eq!(app.active_session.get_attempted_words(), 
			vec!["The", "quick"]
		);

		app.on_del();
		app.on_del();
		assert_eq!(app.active_session.input[1], "d");
		app.on_del();
		app.on_del();
		app.on_del();
		app.on_del();
		assert_eq!(app.active_session.input[0], "a");
		assert_eq!(app.active_session.input.len(), 1);
		app.on_del();
		app.on_del();
		app.on_del();
		assert_eq!(app.active_session.input[0], "");
		assert_eq!(app.active_session.input.len(), 1);	
		
		assert_eq!(app.active_session.get_input_words().len(), 
			app.active_session.get_attempted_words().len());
		
		assert_eq!(app.active_session.get_input_words(), 
			vec![""]
		);
		assert_eq!(app.active_session.get_attempted_words(), 
			vec!["The"]
		);
	}


	#[test]
	fn test_2() {
		let mut app = App::default();
		app.open_typing();
		for c in "The quick brown fox jumps over the lazy dog".chars() {
			if c == ' '{
				app.on_space();
			}
			else {
				app.on_key(c);
			}
		}
		assert_eq!(app.active_session.state, SessionState::Finished);
		assert_eq!(app.state, AppState::Stats);		

		assert_eq!(app.active_session.get_input_words(), 
			vec!["The","quick","brown","fox","jumps","over","the","lazy","dog"]
		);
		assert_eq!(app.active_session.get_attempted_words(), 
			vec!["The","quick","brown","fox","jumps","over","the","lazy","dog"]
		);

		let acc_difference = (app.active_stats.acc - 1.0).abs();
		assert!(acc_difference < 0.0001);
	}
	
	#[test]
	fn test_3() {
		let mut session = Session::from("a b cd".to_string());

		assert_eq!(session.target_words.get_word_at(0),
			Some("a".to_string()));
		assert_eq!(session.target_words.get_word_at(1),
			Some("b".to_string()));
		assert_eq!(session.target_words.get_word_at(2),
			Some("cd".to_string()));

		session.on_char('a');
		session.on_space();
		session.on_char('x');
		session.on_space();
		session.on_char('c');
		session.on_char('d');

		assert_eq!(session.input, 
			vec![
				"a".to_string(),
				"x".to_string(),
				"cd".to_string()
			]);

		assert_eq!(session.state, SessionState::Finished);

		let stats = SessionStats::from(&session);
		
		// MonkeyType will yield 2/3 accuracy in this situation
		let target_acc: f32 = 4.0 / 5.0;

		let acc_difference = (stats.acc - target_acc).abs();
		assert!(
			acc_difference < 0.0001, "Accuracy was wrong ({} vs {})", 
			stats.acc, 
			target_acc
		);
		
		// These are based on results from MonkeyType
		assert_eq!(stats.char_corr, 4);
		assert_eq!(stats.char_total, 5);
		assert_eq!(stats.word_corr, 2);
		assert_eq!(stats.word_total, 3);
	}

	// TODO tests for wpm, wpm_raw
}
