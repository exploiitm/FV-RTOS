use core::cell::RefCell;

use crate::StatusType;
use critical_section::Mutex;
use defmt::info;

const NUM_ALARMS: usize = 5;

pub type TickType = i32;
pub type TickRefType = *mut TickType;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AlarmBaseType {
    pub maxallowedvalue: TickType, // Maximum possible allowed count value in ticks
    pub ticksperbase: TickType,    // Number of ticks required to reach counter specific unit
    mincycle: TickType,
}

impl AlarmBaseType {
    pub const fn defaultinit() -> Self {
        AlarmBaseType {
            maxallowedvalue: 10,
            ticksperbase: 1,
            mincycle: 10,
        }
    }
}

pub type AlarmBaseRefType = *mut AlarmBaseType;

pub type AlarmType = usize;

//TODO : Add an option
#[derive(Clone, Copy, Debug)]
pub struct Alarm {
    pub alarm_base: AlarmBaseType,
    pub tick: TickType,
    pub h_ticks: i32,
    pub cycle: i32,
    pub active: bool,
}

pub static SOFTW_ALARMS: Mutex<RefCell<[Alarm; NUM_ALARMS]>> = Mutex::new(RefCell::new(
    [Alarm {
        alarm_base: AlarmBaseType::defaultinit(),
        tick: 0,
        h_ticks: 0,
        cycle: 0,
        active: false,
    }; NUM_ALARMS],
));

/// Reads the alarm base characteristics.
///
/// # Syntax
/// ```ignore
/// StatusType GetAlarmBase(AlarmType AlarmID, AlarmBaseRefType Info)
/// ```
///
/// # Parameters
///
/// * `alarm_id` (in) — Reference to the alarm.
/// * `info` (out) — Reference to a structure with constants of the alarm base.
///
/// # Description
///
/// This service reads the alarm base characteristics.
/// The return value `info` is a structure of type [`AlarmBaseType`]
/// that contains the alarm base information.
///
/// # Particularities
///
/// Allowed on task level, ISR, and in several hook routines
///
/// # Status
///
/// * **Standard:**
///   * `E_OK` — No error.
/// * **Extended:**
///   * `E_OS_ID` — Alarm `alarm_id` is invalid.
#[unsafe(no_mangle)]
pub extern "C" fn GetAlarmBase(alarm_id: AlarmType, info: AlarmBaseRefType) -> StatusType {
    if alarm_id >= NUM_ALARMS {
        #[cfg(debug_assertions)]
        info!("Invalid Alarm ID");
        return StatusType::EOsId;
    }
    critical_section::with(|cs| {
        let alarm_ref = &SOFTW_ALARMS.borrow_ref(cs)[alarm_id];
        unsafe {
            *info = alarm_ref.alarm_base;
        }
        StatusType::EOk
    })
}

/// Returns the relative value in ticks before an alarm expires.
///
/// # Syntax
/// ```ignore
/// StatusType GetAlarm(AlarmType AlarmID, TickRefType Tick)
/// ```
///
/// # Parameters
///
/// * `alarm_id` (in) — Reference to an alarm.
/// * `tick` (out) — Relative value in ticks before the alarm `alarm_id` expires.
///   If the alarm is not in use, the value of `tick` is undefined.
///
/// # Description
///
/// This service returns the relative value in ticks before the alarm `alarm_id`
/// expires.
///
/// # Particularities
///
/// * If `alarm_id` is not in use, `tick` is undefined.
/// * Allowed on task level, ISR, and in several hook routines.
///
/// # Status
///
/// * **Standard:**
///   * `E_OK` — No error.
///   * `E_OS_NOFUNC` — Alarm `alarm_id` is not used.
///
#[unsafe(no_mangle)]
pub extern "C" fn GetAlarm(alarm_id: AlarmType, tick: TickRefType) -> StatusType {
    if alarm_id >= NUM_ALARMS {
        #[cfg(debug_assertions)]
        info!("Invalid Alarm ID");

        return StatusType::EOsId;
    }
    critical_section::with(|cs| {
        let alarm_ref = &SOFTW_ALARMS.borrow_ref_mut(cs)[alarm_id];
        if !alarm_ref.active {
            return StatusType::EOsNoFunc;
        }
        unsafe { *tick = alarm_ref.alarm_base.maxallowedvalue - alarm_ref.tick };
        StatusType::EOk
    })
}

//TODO:: design cyclic alarms.
#[unsafe(no_mangle)]
pub extern "C" fn SetRelAlarm(
    alarm_id: AlarmType,
    increment: TickType,
    cycle: TickType,
) -> StatusType {
    if alarm_id >= NUM_ALARMS {
        #[cfg(debug_assertions)]
        info!("Invalid Alarm ID");

        return StatusType::EOsId;
    }
    critical_section::with(|cs| {
        let mut alarms = SOFTW_ALARMS.borrow_ref_mut(cs);
        let alarm_ref = &mut alarms[alarm_id];

        if alarm_ref.active {
            #[cfg(debug_assertions)]
            info!("Alarm Already in use");
            return StatusType::EOsState;
        }
        if increment > alarm_ref.alarm_base.maxallowedvalue || increment <= 0 {
            #[cfg(debug_assertions)]
            info!("Increment is invalid");
            return StatusType::EOsValue;
        }

        if cycle != 0
            && (cycle < alarm_ref.alarm_base.mincycle
                || cycle > alarm_ref.alarm_base.maxallowedvalue)
        {
            return StatusType::EOsValue;
        }

        alarm_ref.tick = alarm_ref.alarm_base.maxallowedvalue - increment;
        alarm_ref.active = true;

        alarm_ref.cycle = if cycle != 0 {
            alarm_ref.alarm_base.maxallowedvalue - cycle
        } else {
            0
        };
        StatusType::EOk
    })
}
// Sets an alarm at an absolute tick value.
///
/// # Parameters
/// * `alarm_id` - Reference to the alarm element.
/// * `start` - Absolute value in ticks when the alarm should expire.
/// * `cycle` - Cycle value for cyclic alarms. Must be `0` for single alarms.
///
/// # Behavior
/// - Occupies the alarm specified by `alarm_id`.
/// - When the counter reaches `start`, the assigned task is activated, the assigned
///   event (for extended tasks) is set, or the alarm callback routine is called.
/// - If `start` is very close to the current counter value, the alarm may expire
///   immediately, possibly before the function returns.
/// - If `start` was already passed before the system call, the alarm will expire
///   only after the counter overflows and reaches `start` again.
/// - If `cycle` is nonzero, the alarm is automatically re-scheduled after expiry
///   with a relative value of `cycle`.
///
/// # Particularities
/// - The alarm must not already be in use. To change values of an active alarm,
///   cancel it first with `CancelAlarm`.
/// - Allowed at task level and in ISRs, but not in hook routines.
///
/// # Return Values
/// * `E_OK` - No error.
/// * `E_OS_STATE` - Alarm already in use.
/// * `E_OS_ID` - Invalid `alarm_id`.
/// * `E_OS_VALUE` -
///   - `start` is outside counter limits (`< 0` or `> maxallowedvalue`).
///   - `cycle` is nonzero but outside counter limits (`< mincycle` or `> maxallowedvalue`).
///
#[unsafe(no_mangle)]
pub extern "C" fn SetAbsAlarm(alarm_id: AlarmType, start: TickType, cycle: TickType) -> StatusType {
    if alarm_id >= NUM_ALARMS {
        return StatusType::EOsId;
    }
    critical_section::with(|cs| {
        let mut alarms = SOFTW_ALARMS.borrow_ref_mut(cs);
        let alarm_ref = &mut alarms[alarm_id];

        if alarm_ref.active {
            return StatusType::EOsState;
        }

        if start > alarm_ref.alarm_base.maxallowedvalue {
            return StatusType::EOsValue;
        }

        if cycle != 0
            && (cycle < alarm_ref.alarm_base.mincycle
                || cycle > alarm_ref.alarm_base.maxallowedvalue)
        {
            return StatusType::EOsValue;
        }

        alarm_ref.cycle = if cycle != 0 {
            alarm_ref.alarm_base.maxallowedvalue - cycle
        } else {
            0
        };
        alarm_ref.tick = start;
        alarm_ref.active = true;

        StatusType::EOk
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn CancelAlarm(alarm_id: AlarmType) -> StatusType {
    if alarm_id >= NUM_ALARMS {
        return StatusType::EOsId;
    }
    critical_section::with(|cs| {
        let alarms = &SOFTW_ALARMS.borrow_ref_mut(cs);
        let mut alarm_ref = alarms[alarm_id];
        if !alarm_ref.active {
            return StatusType::EOsNoFunc;
        }
        alarm_ref.active = false;
        alarm_ref.tick = 0;
        StatusType::EOk
    })
}

unsafe extern "C" {
    // hardcoding alarm callbacks until OIL parser is ready
    pub fn AlarmCallback0();
    pub fn AlarmCallback1();
    pub fn AlarmCallback2();
    pub fn AlarmCallback3();
    pub fn AlarmCallback4();
}
