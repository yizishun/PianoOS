
pub const HART_INFO_SIZE: usize = size_of::<HartContext>();

#[repr(C, align(128))]
pub struct HartContext {
        flow_context: crate::arch::common::FlowContext,
        hartid: usize,
}

impl HartContext {
        pub fn init(&mut self, hartid: usize){
                self.hartid = hartid;
        }

        pub fn get_hartnum() -> usize {
                crate::PLATFORM.get().unwrap().board_info.cpu_num.unwrap()
        }

        pub fn get_hartid(&self) -> usize {
                self.hartid
        }
}

pub enum HartState {
        Started,
        Stoped,
        StartPeding,
        StopPeding,
        Invalid,
}

impl core::fmt::Display for HartState {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                        HartState::Started => write!(f, "Hart is Started"),
                        HartState::Stoped => write!(f, "Hart is Stoped"),
                        HartState::StartPeding => write!(f, "Hart is Start pending"),
                        HartState::StopPeding => write!(f, "Hart is Stop pending"),
                        HartState::Invalid => write!(f, "Hart is Invalid"),
                }
        }
}