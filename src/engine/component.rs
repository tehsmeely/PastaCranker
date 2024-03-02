use crate::core_elements::{CoreParameters, CoreState, GameMode};

pub trait Component {
    fn execute(&mut self, state: &CoreState, parameters: &CoreParameters);

    fn init(&mut self, state: &CoreState, parameters: &CoreParameters);

    fn priority(&self) -> u8;

    fn should_execute(&self, mode: GameMode) -> bool;

    fn on_mode_change(&mut self, mode: GameMode);
}
