mod bi_map;
mod hacking_puzzle;

use hacking_puzzle::{HackingPuzzle, GuessResult};

use std::collections::VecDeque;
use std::io::{self, stdout};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

use ratatui::{crossterm::{event::{self, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand}, prelude::*, widgets::Paragraph};

fn main() -> io::Result<()> {
	enable_raw_mode()?;
	stdout().execute(EnterAlternateScreen)?;
	let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

	let mut puzzle = HackingPuzzle::generate(0);
	let mut puzzle_done = false;
	let mut current_text_entry = String::with_capacity(24);
	let mut entered_commands_buffer = VecDeque::<String>::with_capacity(16);

	let show_cursor = Arc::new(Mutex::new(false));
	let show_cursor_thread = show_cursor.clone();
	let show_lockout_message = Arc::new(Mutex::new(false));
	let show_lockout_message_thread = show_lockout_message.clone();


	thread::spawn(move || {
		loop {
			thread::sleep(Duration::from_millis(600));
			let mut show_cursor_value = show_cursor_thread.lock().unwrap();
			*show_cursor_value ^= true;
		}
	});

	thread::spawn(move || {
		loop {
			thread::sleep(Duration::from_millis(500));
			let mut show_lockout_value = show_lockout_message_thread.lock().unwrap();
			*show_lockout_value ^= true;
		}
	});

	let mut should_quit = false;
	while !should_quit {
		terminal.draw(|frame| {
			macro_rules! xy {
				($x:expr, $y:expr) => {
					Rect::new($x, $y, frame.size().width, frame.size().height)
				};
			}

			frame.render_widget(
				Paragraph::new(Text::raw("ROBCO INDUSTRIES (TM) TERMLINK PROTOCOL")),
				xy!(0, 0),
			);

			frame.render_widget(
				Paragraph::new(if puzzle.get_guesses_left() > 1 { 
					Text::raw("ENTER PASSWORD NOW")
				} else if *show_lockout_message.lock().unwrap() && puzzle.get_guesses_left() == 1 {
					Text::raw("!!! WARNING: LOCKOUT IMMINENT !!!")
				} else {
					Text::raw(" ".repeat(frame.size().width as usize))
				}),
				xy!(0, 1),
			);

			frame.render_widget(
				Paragraph::new(Text::raw(format!("{} ATTEMPT(S) LEFT: {}", puzzle.get_guesses_left(), "■ ".repeat(puzzle.get_guesses_left())))),
				xy!(0, 3),
			);

			frame.render_widget(
				Paragraph::new(Text::raw(puzzle.get_full_terminal_text())),
				xy!(0, 5),
			);

			frame.render_widget(
				Paragraph::new(Text::raw(format!("> {}{}", current_text_entry.to_uppercase(), if *show_cursor.lock().unwrap() { "■" } else { "" }))),
				xy!(40, 21),
			);

			for (index, entry) in entered_commands_buffer.iter().rev().enumerate() {
				frame.render_widget(
					Paragraph::new(Text::raw(format!("> {}", entry))),
					xy!(40, (19 - index) as u16),
				)
			}
		})?;

		if event::poll(Duration::from_millis(50))? {
			if let Event::Key(key) = event::read()? {
				if key.kind == event::KeyEventKind::Press || key.kind == event::KeyEventKind::Repeat {
					if puzzle_done {
						should_quit = true;
					}

					match key.code {
						KeyCode::Char(chr) => {
							if current_text_entry.len() < 24 {
								current_text_entry.push(chr);
							}
						},

						KeyCode::Backspace => {
							if !current_text_entry.is_empty() {
								current_text_entry.pop();
							}
						},

						KeyCode::Enter => {
							let entry_trimmed = current_text_entry.trim_end();
							let guess = entry_trimmed.to_lowercase();
							let mut command_entry = vec![entry_trimmed.to_uppercase()];
							if guess == "quit" {
								should_quit = true;
							} else {
								match puzzle.guess_word(&guess) {
									GuessResult::Correct => {
										command_entry.extend(vec!["Exact match!".into(), "Please wait".into(), "while system".into(), "is accessed.".into()]);
										puzzle_done = true;
									},
						
									GuessResult::WrongWord(letters_correct, letters_total) => {
										if puzzle.get_guesses_left() > 0 {
											command_entry.extend(vec!["Entry denied".into(), format!("{}/{} correct.", letters_correct, letters_total)]);
											if puzzle.get_guesses_left() == 1 {
												*show_lockout_message.lock().unwrap() = true;
											}
										} else {
											command_entry.extend(vec!["Entry denied".into(), "Lockout in\n  progress.".into()]);
											puzzle_done = true;
										}
									},
						
									GuessResult::FoundBracketSequence(allowance_replenished) => {
										if allowance_replenished {
											command_entry.push("Allowance replenished.".into());
										} else {
											command_entry.push("Dud removed.".into());
										}
									},
						
									_ => {
										command_entry.push("Entry denied.".into());
									},
								}
							}

							current_text_entry.clear();
							for line in command_entry {
								entered_commands_buffer.push_back(line);
								if entered_commands_buffer.len() > 15 {
									entered_commands_buffer.pop_front();
								}
							}
						},

						_ => {},
					}

					*show_cursor.lock().unwrap() = true;
				}
			}
		}
	}

	disable_raw_mode()?;
	stdout().execute(LeaveAlternateScreen)?;

	Ok(())
}
