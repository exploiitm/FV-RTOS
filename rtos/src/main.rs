#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp235x_hal::{self as hal, entry};
use rtos_core;


/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[entry]
fn main() -> ! {
    info!("Program start");

    // External high-speed crystal on the pico board is 12Mhz

    rtos_core::start_os();
}
