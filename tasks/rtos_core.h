#ifndef RTOS_CORE_H
#define RTOS_CORE_H

#pragma once

typedef enum StatusType {
  EOk = 0,
} StatusType;

extern void Task1(void);

enum StatusType ActivateTask(unsigned int tid);

void TerminateTask(unsigned int tid);

#endif  /* RTOS_CORE_H */
