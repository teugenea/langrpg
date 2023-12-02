use std::ops::ControlFlow;
use crate::command::Command;
use crate::states::{FightState, OnMapState, State, StateData};

pub struct StateDispatcher {
    states: Vec<Box<dyn State>>,
}

impl StateDispatcher {
    pub fn new() -> Self {
        let mut states: Vec<Box<dyn State>> = Vec::new();
        states.push(Box::new(OnMapState {}));
        Self {
            states
        }
    }

    pub fn dispatch_command(&mut self, cmd: Command) {
        if let Some(state) = self.states.last_mut() {
            match state.run_command(cmd) {
                Ok(data) => {
                    self.process_data(data);
                }
                Err(_) => {}
            }
        };
    }

    fn process_data(&mut self, data: ControlFlow<StateData, StateData>) -> StateData {
        match data {
            ControlFlow::Continue(dt) => {
                if let Some(new_state) = switch_state_if_needed(&dt) {
                    let result = new_state.get_data();
                    self.states.push(new_state);
                    return result;
                }
                dt
            }
            ControlFlow::Break(dt) => {
                if let Some(new_state) = switch_state_if_needed(&dt) {
                    let result = new_state.get_data();
                    self.states.pop();
                    self.states.push(new_state);
                    return result;
                }
                self.states.pop();
                dt
            }
        }
    }
}

fn switch_state_if_needed(data: &StateData) -> Option<Box<dyn State>> {
    match data {
        StateData::Map { is_encounter } => {
            if *is_encounter {
                return Some(Box::new(FightState{}));
            }
            None
        }
        StateData::Fight => None
    }
}