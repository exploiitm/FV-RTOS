#![no_std]

// Use C-compatible core types
use core::ffi::c_void;

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

mod alarms;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    EOk = 0,
    EOsId = 1,
    EOsNoFunc = 2,
    EOsValue = 3,
    EOsState = 4,
}

pub fn start_os() -> ! {
    ActivateTask();
    info!("Control back to start_os");
    loop {}
}

unsafe extern "C" {
    fn Task1() -> c_void;
}

#[unsafe(no_mangle)]
pub extern "C" fn ActivateTask() -> StatusType {
    // TODO: lookup & enqueue in your scheduler
    info!("Task Activated ");
    unsafe {
        Task1();
    }
    StatusType::EOk
}

#[unsafe(no_mangle)]
pub extern "C" fn TerminateTask() {
    // TODO: context switch out
    info!("Task Terminated ");
}
