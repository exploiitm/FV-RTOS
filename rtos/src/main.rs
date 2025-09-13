#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp235x_hal::{self as hal, entry, fugit::MicrosDurationU32};
use rtos_core;

mod board;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[entry]
fn main() -> ! {
    info!("Program start");

    board::init();
    board::set_alarm(MicrosDurationU32::secs(5));
    rtos_core::start_os();
}
