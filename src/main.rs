mod bi_map;
mod hacking_puzzle;

use std::io;
//use ratatui::{crossterm::{event::{self, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand}, prelude::*, widgets::{Block, Paragraph}};

use hacking_puzzle::{HackingPuzzle, GuessResult};

fn main() -> io::Result<()> {
	/*enable_raw_mode()?;
	stdout().execute(EnterAlternateScreen)?;
	let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

	let mut should_quit = false;
	while !should_quit {
		terminal.draw(ui)?;
		should_quit = handle_events()?;
	}

	disable_raw_mode()?;
	stdout().execute(LeaveAlternateScreen)?;*/

	/*let mut max = 396;
	let mut rng = thread_rng();
	let normal_distr = Normal::new(WORD_DISTANCE_MEAN, WORD_DISTNACE_STDDEV).unwrap();
	for _ in 0..16 {
		println!("{}", (normal_distr.sample(&mut rng).round() as usize).clamp(7, 55));
	}
	/*while max > 0 {
		let midpoint = max / 2;
		let index = rng.gen_range(midpoint - 22..=midpoint + 22);
		println!("{}", index);
		max = index;
	}*/*/

	let mut puzzle = HackingPuzzle::generate(0);
	println!("{}", puzzle.get_full_terminal_text());

	while puzzle.get_guesses_left() > 0 {
		let mut input = String::with_capacity(10);
		io::stdin().read_line(&mut input).unwrap();
		match puzzle.guess_word(&input.trim_end().to_lowercase()) {
			GuessResult::Correct => {
				println!("Exact match!");
				return Ok(());
			},

			GuessResult::WrongWord(letters_correct, letters_total) => {
				println!("Entry denied\n{}/{} correct.\n", letters_correct, letters_total);	
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

	Ok(())
}

/*fn handle_events() -> io::Result<bool> {
	if event::poll(std::time::Duration::from_millis(50))? {
		if let Event::Key(key) = event::read()? {
			if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
				return Ok(true);
			}
		}
	}
	
	Ok(false)
}

fn ui(frame: &mut Frame) {
	frame.render_widget(
		Paragraph::new(Text::raw("Hello\nThere\nThis\nIs\nA\nTest")),//.block(Block::bordered().title("Greeting")),
		frame.size(),
	);
}*/
