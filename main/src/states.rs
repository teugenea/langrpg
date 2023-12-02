use std::fmt::{Debug, Display};
use std::ops::ControlFlow;

use crate::command::Command;

pub enum StateData {
    Map {
        is_encounter: bool
    },
    Fight
}

pub trait State: Send {
    fn run_command(&mut self, cmd: Command) -> Result<ControlFlow<StateData, StateData>, String>;
    fn get_data(&self) -> StateData;
}

pub struct FightState {

}

impl State for FightState {

    fn run_command(&mut self, cmd: Command) -> Result<ControlFlow<StateData, StateData>, String> {
        println!("Got command in fight state");
        Ok(ControlFlow::Continue(StateData::Fight))
    }

    fn get_data(&self) -> StateData {
        StateData::Fight
    }
}

pub struct OnMapState {

}

impl OnMapState {

}

impl State for OnMapState {

    fn run_command(&mut self, cmd: Command) -> Result<ControlFlow<StateData, StateData>, String> {
        Ok(ControlFlow::Continue(StateData::Map {
            is_encounter: true
        }))
    }

    fn get_data(&self) -> StateData {
        StateData::Map {
            is_encounter: true
        }
    }
}