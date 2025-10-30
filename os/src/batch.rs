use log::info;
use spin::Mutex;

use crate::arch::common::ArchMem;
use crate::global::ARCH;
use crate::arch::common::ArchPower;
use crate::config::MAX_APP_NUM;
use crate::global::_num_app;
pub struct AppManager {
        num_app: usize,
        current_app: Mutex<usize>, //TODO: UnsafeCell?
        app_start_addr: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
        pub fn new() -> Self {
                let num_app_ptr: *const usize = core::ptr::addr_of!(_num_app);
                let num_app_usize: usize = unsafe { *num_app_ptr };
                let count: usize = num_app_usize + 1;
                let app_start_addr_raw: &[usize] =
                        unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), count) };
                let mut app_start_addr: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
                app_start_addr[..count].copy_from_slice(app_start_addr_raw);
                AppManager { num_app: num_app_usize,
                             current_app: Mutex::new(0),
                             app_start_addr: app_start_addr }
        }

        pub fn print_app_info(&self) {
                info!("Kernel app number: {}", self.num_app);
                for i in 0..self.num_app {
                        info!("app {i}: [{:<10p}, {:<10p}]",
                              self.app_start_addr[i] as *const usize,
                              self.app_start_addr[i + 1] as *const usize);
                }
        }

        pub fn get_cur_app(&self) -> usize {
                *self.current_app.lock()
        }

        pub fn move_to_next_app(&self) {
                *self.current_app.lock() += 1;
        }

        pub fn load_app(&self, app_id: usize) {
                use crate::config::APP_BASE_ADDR;
                if app_id > self.num_app {
                        info!("All applications completed! Kennel shutdown");
			ARCH.shutdown(false);
                }
                let app_addr_start = *self.app_start_addr.get(app_id).unwrap();
                let app_addr_end = *self.app_start_addr.get(app_id + 1).unwrap();
                let count = app_addr_end - app_addr_start;
                let dst = APP_BASE_ADDR as *mut u8;
                info!("Kernel loading app({app_id})");
                unsafe {
                        core::ptr::copy_nonoverlapping(app_addr_start as *const u8, dst, count);
                        unsafe {
			    ARCH.fencei();
			}
                }
        }
}
