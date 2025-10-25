use spin::Once;
use crate::platform::Platform;
use crate::config::NUM_HART_MAX;
use crate::arch::hart::HartInfo;
use crate::batch::AppManager;

unsafe extern "C" {
    pub static skernel: usize;
    pub static stext: usize;
    pub static etext: usize;
    pub static srodata: usize;
    pub static erodata: usize;
    pub static sdata: usize;
    pub static edata: usize;
    pub static sstack: usize;
    pub static estack: usize;
    pub static sheap: usize;
    pub static eheap: usize;
    pub static sbss: usize;
    pub static ebss: usize;
    pub static ekernel: usize;

    pub static _num_app: usize;
}


pub static PLATFORM: Once<Platform> = Once::new();

pub static HART_INFO: Once<[HartInfo; NUM_HART_MAX]> = Once::new();

pub static APP_MANAGER: Once<AppManager> = Once::new();