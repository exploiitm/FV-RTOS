use core::cell::RefCell;

use critical_section::Mutex;
use defmt::info;
use rp235x_hal::{
    self as hal,
    fugit::MicrosDurationU32,
    pac::interrupt,
    timer::{Alarm, Alarm0, CopyableTimer0},
};
use rtos_core::alarms::*;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;
static ALARMS: Mutex<RefCell<Option<Alarm0<CopyableTimer0>>>> = Mutex::new(RefCell::new(None));

pub fn init() {
    let mut pac = hal::pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);
    critical_section::with(|cs| {
        let alarm = timer.alarm_0().unwrap();
        ALARMS.borrow(cs).replace(Some(alarm));
    });
}

pub fn set_alarm(time: MicrosDurationU32) {
    critical_section::with(|cs| {
        if let Some(alarm) = ALARMS.borrow_ref_mut(cs).as_mut() {
            alarm.schedule(time).unwrap();
            alarm.enable_interrupt();
        }
    });
    unsafe {
        cortex_m::peripheral::NVIC::unmask(hal::pac::Interrupt::TIMER0_IRQ_0);
    }
}

#[interrupt]
fn TIMER0_IRQ_0() {
    critical_section::with(|cs| {
        #[cfg(debug_assertions)]
        info!("Interrupt !");
        if let Some(alarm) = ALARMS.borrow_ref_mut(cs).as_mut() {
            alarm.clear_interrupt();

            let mut alarms = rtos_core::alarms::SOFTW_ALARMS.borrow_ref_mut(cs);

            for (i, alarm) in alarms.iter_mut().enumerate() {
                if alarm.active {
                    #[cfg(debug_assertions)]
                    info!("Alarm {} is active", i);
                    alarm.tick += alarm.alarm_base.ticksperbase;
                    if alarm.tick > alarm.alarm_base.maxallowedvalue {
                        unsafe {
                            match i {
                                1 => AlarmCallback1(),
                                2 => AlarmCallback2(),
                                3 => AlarmCallback3(),
                                4 => AlarmCallback4(),
                                5 => AlarmCallback5(),
                                _ => (),
                            }
                        }
                        alarm.tick = 0;
                    }
                }
            }

            //TODO : Change this hardcoded value
            let _ = alarm.schedule(MicrosDurationU32::secs(1));
            alarm.enable_interrupt();
        }
    })
}
