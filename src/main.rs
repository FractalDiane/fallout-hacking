mod bi_map;
mod hacking_puzzle;

use std::io::{self, stdout};
use ratatui::{crossterm::{event::{self, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand}, prelude::*, widgets::Paragraph};

use hacking_puzzle::{HackingPuzzle, GuessResult};

fn main() -> io::Result<()> {
	enable_raw_mode()?;
	stdout().execute(EnterAlternateScreen)?;
	let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

	let mut puzzle = HackingPuzzle::generate(0);
	let mut current_text_entry = String::with_capacity(24);

	let mut should_quit = false;
	while !should_quit {
		terminal.draw(|frame| {
			frame.render_widget(
				Paragraph::new(Text::raw("ROBCO INDUSTRIES (TM) TERMLINK PROTOCOL")),
				Rect::new(0, 0, frame.size().width, frame.size().height),
			);

			frame.render_widget(
				Paragraph::new(Text::raw(format!("{} ATTEMPT(S) LEFT: {}", puzzle.get_guesses_left(), "■ ".repeat(puzzle.get_guesses_left())))),
				Rect::new(0, 3, frame.size().width, frame.size().height),
			);

			frame.render_widget(
				Paragraph::new(Text::raw(puzzle.get_full_terminal_text())),
				Rect::new(0, 5, frame.size().width, frame.size().height),
			);

			frame.render_widget(
				Paragraph::new(Text::raw(format!("> {}", current_text_entry))),
				Rect::new(40, 21, frame.size().width, frame.size().height),
			);
		})?;

		if event::poll(std::time::Duration::from_millis(50))? {
			if let Event::Key(key) = event::read()? {
				if key.kind == event::KeyEventKind::Press || key.kind == event::KeyEventKind::Repeat {
					match key.code {
						KeyCode::Char(chr) => {
							if current_text_entry.len() < 24 {
								current_text_entry.push(chr);
							}
						},

						KeyCode::Backspace => {
							if !current_text_entry.is_empty() {
								current_text_entry.pop().unwrap();
							}
						},

						KeyCode::Enter => {
							let guess = current_text_entry.trim_end().to_lowercase();
							if guess == "quit" {
								should_quit = true;
							} else {
								match puzzle.guess_word(&guess) {
									GuessResult::Correct => {
										println!("Exact match!");
										return Ok(());
									},
						
									GuessResult::WrongWord(letters_correct, letters_total) => {
										println!("Entry denied\n{}/{} correct.\n", letters_correct, letters_total);
										if puzzle.get_guesses_left() == 0 {
											should_quit = true;
										}
									},
						
									GuessResult::FoundBracketSequence(allowance_replenished) => {
										if allowance_replenished {
											println!("Allowance replenished.");
										} else {
											println!("Dud removed.");
										}
									},
						
									_ => {
										println!("Invalid");
									},
								}
							}

							current_text_entry.clear();
						},

						_ => {},
					}
				}
			}
		}
	}

	disable_raw_mode()?;
	stdout().execute(LeaveAlternateScreen)?;

	Ok(())
}
