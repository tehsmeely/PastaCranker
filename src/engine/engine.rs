use crate::engine::component::Component;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Debug, Display};
use core::hash::Hash;
use hashbrown::HashMap;
use crate::core_elements::{CoreParameters, CoreState, GameMode};

trait Key: Debug + Display + Copy + Eq + Hash {}
pub struct Engine<K> {
    state: CoreState,
    params: CoreParameters,
    component_map: HashMap<K, Box<dyn Component>>,
    execution_key_order: Vec<K>,
    current_mode: GameMode,
}

impl<K: Key> Engine<K> {
    pub fn new(state: CoreState, params: CoreParameters) -> Self {
        Self {
            state,
            params,
            component_map: HashMap::new(),
            execution_key_order: Vec::new(),
            current_mode: GameMode::LevelSelect,
        }
    }

    pub fn add_component(&mut self, key: K, component: Box<dyn Component>) {
        component.init(&self.state, &self.params);
        self.component_map.insert(key, component);
        self.execution_key_order.push(key);
    }

    pub fn regenerate_execution_order(&mut self) {
        let mut keys_with_priority: Vec<(K, u8)> = self
            .component_map
            .iter()
            .map(|(key, component)| (*key, component.priority()))
            .collect();
        keys_with_priority.sort_by_key(|(_, priority)| *priority);
        self.execution_key_order = keys_with_priority.iter().map(|(key, _)| *key).collect();
    }

    pub fn execute(&mut self) {
        for key in &self.execution_key_order {
            if let Some(component) = self.component_map.get_mut(key) {
                component.execute(self.);
            }
        }
    }
}
