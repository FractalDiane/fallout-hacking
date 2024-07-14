use std::collections::HashMap;
use std::hash::Hash;

pub struct BiMap<L: Hash + Eq + Clone, R: Hash + Eq + Clone> {
	map_l: HashMap<L, R>,
	map_r: HashMap<R, L>,
}

impl<L: Hash + Eq + Clone, R: Hash + Eq + Clone> BiMap<L, R> {
	pub fn with_capacity(capacity: usize) -> Self {
		BiMap{
			map_l: HashMap::<L, R>::with_capacity(capacity),
			map_r: HashMap::<R, L>::with_capacity(capacity),
		}
	}

	pub fn insert(&mut self, key: L, value: R) {
		self.map_l.insert(key.clone(), value.clone());
		self.map_r.insert(value, key);
	}

	pub fn get_left(&self, key: &L) -> Option<&R> {
		self.map_l.get(key)
	}

	pub fn get_right(&self, key: &R) -> Option<&L> {
		self.map_r.get(key)
	}
}
