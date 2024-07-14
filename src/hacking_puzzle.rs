use rand::prelude::*;
use rand_distr::Normal;

use crate::bi_map::BiMap;
use std::collections::{HashMap, HashSet};

const WORD_LISTS: &[&[&[&'static str]]] = &[
	&[
		&["spies", "joins", "tires", "trick", "tried", "skies", "terms", "third", "fries", "price", "tries", "trite", "tanks", "thick", "tribe", "texas"],
	],
];

const JUNK_CHARACTER_COUNT: usize = 29;
const JUNK_CHARACTER_POOL: [u8; JUNK_CHARACTER_COUNT] = *b"()[]{}<>!@#$%^&*-_=+;:'\",./\\?";

const WORD_DISTANCE_MEAN: f32 = 24.3125;
const WORD_DISTNACE_STDDEV: f32 = 16.592041264815;

const LINE_BYTE_COUNT: usize = 12;
const PAGE_LINE_COUNT: usize = 34;
const PAGE_BYTE_COUNT: usize = LINE_BYTE_COUNT * PAGE_LINE_COUNT;

pub struct HackingPuzzle {
	words_list: &'static [&'static str],
	correct_word_index: usize,
	text: String,
	start_address: usize,
	word_indices_in_text: BiMap<&'static str, usize>,

	guesses_left: usize,
	bracket_sequences_remaining: HashMap<String, usize>,
	removed_duds: HashSet<usize>,
}

pub enum GuessResult {
	Correct,
	WrongWord(usize, usize),
	FoundBracketSequence(bool),
	Invalid,
}

impl HackingPuzzle {
	pub fn generate(difficulty: usize) -> Self {
		let mut rng = thread_rng();

		let selected_wordlist = WORD_LISTS[difficulty].choose(&mut rng).unwrap();
		let word_length = selected_wordlist[0].len();
		let mut word_indices_in_text = BiMap::<&'static str, usize>::with_capacity(selected_wordlist.len());
		
		{
			let normal_distr = Normal::new(WORD_DISTANCE_MEAN, WORD_DISTNACE_STDDEV).unwrap();
			let mut current_index = 0;
			for word in *selected_wordlist {
				let distance = (normal_distr.sample(&mut rng).round() as usize).clamp(7, 55);
				word_indices_in_text.insert(word, current_index + distance);
				current_index += distance;
			}
		}
			
		let mut sub_result = String::with_capacity(1024);
		let mut bracket_sequences = HashMap::<String, usize>::with_capacity(24);

		{
			let mut current_bracket_sequence = String::with_capacity(LINE_BYTE_COUNT);
			let mut bracket_indices_used = HashSet::<usize>::with_capacity(LINE_BYTE_COUNT);
			let mut in_open_paren = false;
			let mut in_open_bracket = false;
			let mut in_open_brace = false;
			let mut in_open_angle = false;

			let mut index = 0;
			let mut last_index = 0;
			while index < PAGE_BYTE_COUNT {
				if index / LINE_BYTE_COUNT != last_index / LINE_BYTE_COUNT {
					current_bracket_sequence.clear();
					bracket_indices_used.clear();
					in_open_paren = false;
					in_open_bracket = false;
					in_open_brace = false;
					in_open_angle = false;
				}

				last_index = index;

				if let Some(word) = word_indices_in_text.get_right(&index) {
					sub_result += &word.to_uppercase();
					index += word_length;

					current_bracket_sequence.clear();
					bracket_indices_used.clear();
					in_open_paren = false;
					in_open_bracket = false;
					in_open_brace = false;
					in_open_angle = false;
				} else {
					let junk_char = *JUNK_CHARACTER_POOL.choose(&mut rng).unwrap() as char;
					current_bracket_sequence.push(junk_char);
					match junk_char {
						'(' => {
							in_open_paren = true;
						},
						'[' => {
							in_open_bracket = true;
						},
						'{' => {
							in_open_brace = true;
						},
						'<' => {
							in_open_angle = true;
						},

						')' => {
							if in_open_paren {
								try_add_bracket_sequence('(', ')', &current_bracket_sequence, &mut bracket_sequences, &mut bracket_indices_used);
								in_open_paren = false;
							}
						},
						']' => {
							if in_open_bracket {
								try_add_bracket_sequence('[', ']', &current_bracket_sequence, &mut bracket_sequences, &mut bracket_indices_used);
								in_open_bracket = false;
							}
						},
						'}' => {
							if in_open_brace {
								try_add_bracket_sequence('{', '}', &current_bracket_sequence, &mut bracket_sequences, &mut bracket_indices_used);
								in_open_brace = false;
							}
						},
						'>' => {
							if in_open_angle {
								try_add_bracket_sequence('<', '>', &current_bracket_sequence, &mut bracket_sequences, &mut bracket_indices_used);
								in_open_angle = false;
							}
						},

						_ => {},
					}

					sub_result.push(junk_char);
					index += 1;
				}
			}
		}

		let correct_word_index = rng.gen_range(0..selected_wordlist.len());
		let start_address = rng.gen_range(0x0000..0xffffusize - PAGE_BYTE_COUNT);
		HackingPuzzle{
			text: sub_result,
			start_address,
			words_list: selected_wordlist,
			word_indices_in_text,
			correct_word_index,
			guesses_left: 4,
			bracket_sequences_remaining: bracket_sequences,
			removed_duds: HashSet::with_capacity(10),
		}
	}

	pub fn guess_word(&mut self, word: &str) -> GuessResult {
		if self.word_indices_in_text.get_left(&word).is_some() {
			let correct_word = &self.words_list[self.correct_word_index];
			let mut chars_correct = 0;
			for (guess_char, actual_chr) in word.chars().zip(correct_word.chars()) {
				if guess_char == actual_chr {
					chars_correct += 1;
				}
			}

			if chars_correct == correct_word.len() { 
				GuessResult::Correct
			} else {
				self.guesses_left -= 1;
				GuessResult::WrongWord(chars_correct, correct_word.len())
			}
		} else if *self.bracket_sequences_remaining.get(word).unwrap_or(&0) > 0 {
			*self.bracket_sequences_remaining.entry(word.to_string()).or_default() -= 1;

			if thread_rng().gen_bool(0.2) {
				self.guesses_left = 4;
				return GuessResult::FoundBracketSequence(true);
			} else {
				for _ in 0..100 {
					let removed_dud = thread_rng().gen_range(0..self.words_list.len());
					if removed_dud != self.correct_word_index && self.try_remove_dud(removed_dud) {
						return GuessResult::FoundBracketSequence(false);
					}
				}
				
				for i in 0..self.words_list.len() {
					if i != self.correct_word_index && self.try_remove_dud(i) {
						return GuessResult::FoundBracketSequence(false);
					}
				}
			}

			GuessResult::Invalid
		} else {
			GuessResult::Invalid
		}
	}

	pub fn get_full_terminal_text(&self) -> String {
		let mut result = String::with_capacity(PAGE_BYTE_COUNT * 2);
		for i in 0..PAGE_LINE_COUNT / 2 {
			result += &format!("0x{:04X} {} 0x{:04X} {}\n",
				self.start_address + i * LINE_BYTE_COUNT,
				&self.text[i * LINE_BYTE_COUNT..i * LINE_BYTE_COUNT + LINE_BYTE_COUNT],
				self.start_address + i * LINE_BYTE_COUNT + PAGE_BYTE_COUNT / 2,
				&self.text[i * LINE_BYTE_COUNT + PAGE_BYTE_COUNT / 2..i * LINE_BYTE_COUNT + LINE_BYTE_COUNT + PAGE_BYTE_COUNT / 2],
			);
		}

		result
	}

	pub fn get_guesses_left(&self) -> usize {
		self.guesses_left
	}

	fn try_remove_dud(&mut self, index: usize) -> bool {
		if index != self.correct_word_index && self.removed_duds.insert(index) {
			let word = self.words_list[index];
			let word_index_in_text = self.word_indices_in_text.get_left(&word).unwrap();
			let word_len = self.words_list[0].len();
			self.text.replace_range(word_index_in_text..&(word_index_in_text + word_len), &".".repeat(word_len));

			return true;
		}

		false
	}
}

fn try_add_bracket_sequence(open_bracket: char, close_bracket: char, current_sequence: &String, sequences_map: &mut HashMap<String, usize>, indices_used: &mut HashSet<usize>) {
	if !current_sequence.is_empty() {
		let closing_char = current_sequence.chars().last().unwrap();
		if closing_char == close_bracket {
			for (index, back_chr) in current_sequence.char_indices() {
				if back_chr == open_bracket && !indices_used.contains(&index) {
					let subsequence = current_sequence[index..].to_string();
					*sequences_map.entry(subsequence).or_default() += 1;
					indices_used.insert(index);
				}
			}
		}
	}
}
