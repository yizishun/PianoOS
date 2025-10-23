pub mod entry;
pub mod hart;

pub fn shutdown(fail: bool) -> ! {
    loop {}
}

pub fn sleep(sec: i32) {
  todo!("la sleep");
}