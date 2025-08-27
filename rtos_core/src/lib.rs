#![no_std]

// Use C-compatible core types
use core::ffi::c_uint;

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusType{
    EOk = 0,
}

pub fn start_os()-> ! {
    unsafe {
        Task1();
    }
}

unsafe extern "C" {
    fn Task1() -> !;
}

#[unsafe(no_mangle)]
pub extern "C" fn ActivateTask(tid: c_uint) -> StatusType {
    // TODO: lookup & enqueue in your scheduler
    info!("Task Activated {}", tid);
    StatusType::EOk
}

#[unsafe(no_mangle)]
pub extern "C" fn TerminateTask(tid: c_uint) -> ! {
    // TODO: context switch out
    info!("Task Terminated {}", tid);
    loop {}
}
