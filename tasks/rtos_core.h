#ifndef RTOS_CORE_H
#define RTOS_CORE_H

#pragma once

#include "cstdint.h"

typedef enum StatusType {
  EOk = 0,
  EOsId = 1,
  EOsNoFunc = 2,
  EOsValue = 3,
  EOsState = 4,
} StatusType;

typedef uintptr_t AlarmType;

typedef uint32_t TickType;

typedef struct AlarmBaseType {
  TickType maxallowedvalue;
  TickType ticksperbase;
  TickType _mincycle;
} AlarmBaseType;

typedef struct AlarmBaseType *AlarmBaseRefType;

typedef TickType *TickRefType;

extern void Task1(void);

enum StatusType ActivateTask(void);

void TerminateTask(void);

void print(const char *input);

enum StatusType GetAlarmBase(AlarmType alarm_id, AlarmBaseRefType info);

enum StatusType GetAlarm(AlarmType alarm_id, TickRefType tick);

enum StatusType SetRelAlarm(AlarmType alarm_id, TickType increment, TickType cycle);

enum StatusType SetAbsAlarm(AlarmType alarm_id, TickType start, TickType cycle);

enum StatusType CancelAlarm(AlarmType alarm_id);

#endif  /* RTOS_CORE_H */
