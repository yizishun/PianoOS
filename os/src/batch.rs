use spin::{Mutex, Once};

use crate::config::MAX_APP_NUM;
use crate::global::_num_app;
pub struct AppManager {
    num_app: usize,
    current_app: Mutex<usize>,
    app_start_addr: [usize; MAX_APP_NUM + 1]
}

impl AppManager {
    pub fn new() -> Self {
        let num_app_ptr: *const usize = core::ptr::addr_of!(_num_app);
        let num_app_usize: usize = unsafe { *num_app_ptr }; // 计数
        let count: usize = num_app_usize + 1;
        let app_start_addr_raw: &[usize] = unsafe {
            core::slice::from_raw_parts(num_app_ptr.add(1), count)
        };
        let mut app_start_addr: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
        app_start_addr[..count].copy_from_slice(app_start_addr_raw);
        AppManager {
            num_app: num_app_usize, 
            current_app: Mutex::new(0), 
            app_start_addr: app_start_addr 
        }
    }
}

