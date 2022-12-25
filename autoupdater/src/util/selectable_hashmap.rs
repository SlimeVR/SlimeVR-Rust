use std::{collections::HashMap, fmt::Debug, hash::Hash};

#[derive(Debug)]
pub struct SelectableHashMap<K, V> {
	components: HashMap<K, SelectableThingy<V>>,
}

pub trait Selectable<K> {
	fn dependencies(&self) -> Option<&[K]>;
}

#[derive(Debug)]
struct SelectableThingy<V> {
	enabled: bool,
	enabled_by_dependency: bool,
	value: V,
}

impl<V> SelectableThingy<V> {
	fn new(value: V) -> Self {
		Self {
			enabled: false,
			enabled_by_dependency: false,
			value,
		}
	}

	fn enable(&mut self) {
		self.enabled = true;
	}

	fn disable(&mut self) {
		self.enabled = false;
	}

	fn enable_by_dependency(&mut self) {
		self.enabled_by_dependency = true;
	}

	fn disable_by_dependency(&mut self) {
		self.enabled_by_dependency = false;
	}

	fn is_enabled(&self) -> bool {
		self.enabled || self.enabled_by_dependency
	}
}

impl<K: Hash + Clone + Eq + Debug, V: Selectable<K>> SelectableHashMap<K, V> {
	pub fn new(hashmap: HashMap<K, V>) -> SelectableHashMap<K, V> {
		SelectableHashMap {
			components: hashmap
				.into_iter()
				.map(|(k, v)| (k, SelectableThingy::new(v)))
				.collect(),
		}
	}

	fn enable_inner(&mut self, name: &K, coming_from: Vec<&K>) {
		match self.components.get(name) {
			Some(component) => {
				if component.is_enabled() {
					return;
				}

				let mut component = self.components.remove(&name).unwrap();
				component.enable_by_dependency();

				if let Some(dependencies) = component.value.dependencies() {
					dependencies.iter().for_each(|dep| {
						if coming_from.contains(&dep) {
							panic!("Circular dependency detected: {:?}", coming_from);
						}

						let mut coming_from = coming_from.clone();
						coming_from.push(name);

						self.enable_inner(dep, coming_from);
					});
				}

				self.components.insert(name.clone(), component);
			}
			_ => {}
		}
	}

	fn disable_inner(&mut self, name: &K, coming_from: Vec<&K>) {
		match self.components.get(name) {
			Some(component) => {
				if !component.is_enabled() {
					return;
				}

				let mut component = self.components.remove(name).unwrap();
				component.disable_by_dependency();

				if let Some(dependencies) = component.value.dependencies() {
					dependencies.iter().for_each(|dep| {
						if coming_from.contains(&dep) {
							panic!("Circular dependency detected: {:?}", coming_from);
						}

						let mut coming_from = coming_from.clone();
						coming_from.push(name);

						self.disable_inner(dep, coming_from);
					});
				}

				self.components.insert(name.clone(), component);
			}
			_ => {}
		}
	}

	pub fn disable(&mut self, name: &K) {
		match self.components.get_mut(name) {
			Some(component) => {
				if !component.is_enabled() {
					return;
				}

				let mut component = self.components.remove(&name).unwrap();
				component.disable();

				if let Some(dependencies) = component.value.dependencies() {
					dependencies.iter().for_each(|dep| {
						self.disable_inner(dep, vec![name]);
					});
				}

				self.components.insert(name.clone(), component);
			}
			_ => {}
		}
	}

	pub fn enable(&mut self, name: &K) {
		match self.components.get_mut(name) {
			Some(component) => {
				if component.is_enabled() {
					return;
				}

				let mut component = self.components.remove(&name).unwrap();
				component.enable();

				if let Some(dependencies) = component.value.dependencies() {
					dependencies.iter().for_each(|dep| {
						self.enable_inner(dep, vec![name]);
					});
				}

				self.components.insert(name.clone(), component);
			}
			_ => {}
		}
	}

	pub fn is_enabled(&self, name: &K) -> bool {
		match self.components.get(name) {
			Some(component) => component.is_enabled(),
			_ => false,
		}
	}

	pub fn selected_items_iter(&self) -> impl Iterator<Item = (&K, &V)> {
		self.components
			.iter()
			.filter(|(_, component)| component.is_enabled())
			.map(|(k, component)| (k, &component.value))
	}
}

// impl From<HashMap<String, Component>> for ComponentsToInstall {
// 	fn from(components: HashMap<String, Component>) -> ComponentsToInstall {
// 		let mut to_install = HashMap::new();

// 		for (name, component) in components {
// 			to_install.insert(name, (false, component));
// 		}

// 		ComponentsToInstall {
// 			components: to_install,
// 		}
// 	}
// }
