use core::cell::RefCell;

use crate::StatusType;
use critical_section::Mutex;

const NUM_ALARMS: usize = 5;

pub type TickType = u32;
pub type TickRefType = *mut TickType;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AlarmBaseType {
    pub maxallowedvalue: TickType, // Maximum possible allowed count value in ticks
    pub ticksperbase: TickType,    // Number of ticks required to reach counter specific unit
    _mincycle: TickType,       // TODO:
}

impl AlarmBaseType {
    pub const fn zero() -> Self {
        AlarmBaseType {
            maxallowedvalue: 0,
            ticksperbase: 0,
            _mincycle: 0,
        }
    }
}

pub type AlarmBaseRefType = *mut AlarmBaseType;

pub type AlarmType = usize;

//TODO : Add an option
//TODO:: Add cycle
#[derive(Clone, Copy)]
pub struct Alarm {
    pub alarm_base: AlarmBaseType,
    pub tick: TickType,
    pub active: bool,
}

pub static SOFTW_ALARMS: Mutex<RefCell<[Alarm; NUM_ALARMS]>> = Mutex::new(RefCell::new(
    [Alarm {
        alarm_base: AlarmBaseType::zero(),
        tick: 0,
        active: false,
    }; NUM_ALARMS],
));

#[unsafe(no_mangle)]
pub extern "C" fn GetAlarmBase(alarm_id: AlarmType, info: AlarmBaseRefType) -> StatusType {
    if alarm_id >= NUM_ALARMS {
        return StatusType::EOsId;
    }
    critical_section::with(|cs| {
        let alarm_ref = &SOFTW_ALARMS.borrow_ref(cs)[alarm_id];
        unsafe {
            if !info.is_null() {
                *info = alarm_ref.alarm_base;
            } else {
                return StatusType::EOsId;
            }
        }
        StatusType::EOk
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn GetAlarm(alarm_id: AlarmType, tick: TickRefType) -> StatusType {
    if alarm_id >= NUM_ALARMS {
        return StatusType::EOsId;
    }
    critical_section::with(|cs| {
        let alarm_ref = &SOFTW_ALARMS.borrow_ref_mut(cs)[alarm_id];
        if !alarm_ref.active {
            return StatusType::EOsNoFunc;
        }
        if !tick.is_null() {
            unsafe { *tick = alarm_ref.tick };
        } else {
            return StatusType::EOsNoFunc;
        }
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
        return StatusType::EOsId;
    }
    critical_section::with(|cs| {
        let alarms = &SOFTW_ALARMS.borrow_ref_mut(cs);
        let mut alarm_ref = alarms[alarm_id];
        if alarm_ref.active {
            return StatusType::EOsState;
        }
        if increment == 0 || increment > alarm_ref.alarm_base.maxallowedvalue {
            return StatusType::EOsValue;
        }

        if cycle != 0
            && (cycle < alarm_ref.alarm_base._mincycle
                || cycle > alarm_ref.alarm_base.maxallowedvalue)
        {
            return StatusType::EOsValue;
        }

        let modulo = alarm_ref.alarm_base.maxallowedvalue.wrapping_add(1);
        // Compute expiry = (now + increment) mod (maxallowedvalue+1)
        let when = if modulo != 0 {
            alarm_ref.tick.wrapping_add(increment) % modulo
        } else {
            alarm_ref.tick.wrapping_add(increment)
        };

        alarm_ref.tick = when;
        alarm_ref.active = true;

        //TODO:: implement cyclic alarm.
        StatusType::EOk
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn SetAbsAlarm(alarm_id: AlarmType, start: TickType, cycle: TickType) -> StatusType {
    if alarm_id >= NUM_ALARMS {
        return StatusType::EOsId;
    }
    critical_section::with(|cs| {
        let alarms = &SOFTW_ALARMS.borrow_ref_mut(cs);
        let mut alarm_ref = alarms[alarm_id];

        if alarm_ref.active {
            return StatusType::EOsState;
        }

        if start > alarm_ref.alarm_base.maxallowedvalue {
            return StatusType::EOsValue;
        }

        if cycle != 0
            && (cycle < alarm_ref.alarm_base._mincycle
                || cycle > alarm_ref.alarm_base.maxallowedvalue)
        {
            return StatusType::EOsValue;
        }

        alarm_ref.tick = start;
        alarm_ref.active = true;

        //TODO:: implement cyclic alarm.
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
    pub fn AlarmCallback1();
    pub fn AlarmCallback2();
    pub fn AlarmCallback3();
    pub fn AlarmCallback4();
    pub fn AlarmCallback5();
}
