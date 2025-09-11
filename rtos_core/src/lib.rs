#![no_std]
#![feature(c_variadic)]
// Use C-compatible core types
use core::ffi::{CStr, c_char, c_int, c_void};
use defmt::*;
use defmt_rtt as _;
use embedded_alloc::LlffHeap as Heap;
use panic_probe as _;

extern crate alloc;
use alloc::string::String;
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
#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 1024 * 8;
static mut HEAP_MEM: [core::mem::MaybeUninit<u8>; HEAP_SIZE] =
    [core::mem::MaybeUninit::uninit(); HEAP_SIZE];

pub fn start_os() -> ! {
    unsafe {
        HEAP.init(
            core::ptr::addr_of_mut!(HEAP_MEM).cast::<u8>() as usize,
            HEAP_SIZE,
        );
    }

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

#[unsafe(no_mangle)]
pub extern "C" fn print(input: *const c_char) {
    let c_str = unsafe { CStr::from_ptr(input) };

    let r_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            info!("Invalid Strings");
            return;
        }
    };
    info!("{}", r_str);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn printf(str: *const c_char, mut args: ...) -> c_int {
    use printf_compat::{format, output};
    let mut s = String::new();
    let bytes_written = unsafe { format(str, args.as_va_list(), output::fmt_write(&mut s)) };
    println!("{}", s.as_str());
    bytes_written
}
