use log::error;
use sbi_rt::SbiRet;
use crate::{platform::PLATFORM, print};

pub enum HartState {
    Started,
    Stoped,
    StartPeding,
    StopPeding,
    Invalid
}

impl core::fmt::Display for HartState {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HartState::Started => print!("Hart is Started"),
            HartState::Stoped => print!("Hart is Stoped"),
            HartState::StartPeding => print!("Hart is Start pending"),
            HartState::StopPeding => print!("Hart is Stop pending"),
            HartState::Invalid => print!("Hart is Invalid"),
        }
        Ok(())
    }
}

pub fn get_cur_hartid() -> usize {
    0
}

pub fn get_hartnum() -> usize {
    0
}

pub fn get_cur_hart_state() -> HartState {
    use sbi_rt::{get_marchid, hart_get_status};
    match hart_get_status(get_marchid()).value {
        0 => HartState::Started,
        1 => HartState::Stoped,
        2 => HartState::StartPeding,
        3 => HartState::StopPeding,
        _ => {
            error!("unexpected hart id, may be rustsbi problem"); 
            HartState::Invalid
        }
    }
}

pub fn get_hart_state(id: usize) -> HartState {
    use sbi_rt::hart_get_status;
    match hart_get_status(id).value {
        0 => HartState::Started,
        1 => HartState::Stoped,
        2 => HartState::StartPeding,
        3 => HartState::StopPeding,
        _ => {
            error!("unexpected hart id, may be rustsbi problem"); 
            HartState::Invalid
        }
    }
}