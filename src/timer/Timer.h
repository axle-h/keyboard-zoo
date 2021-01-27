#pragma once

#include <chrono>

typedef std::chrono::steady_clock Clock;
typedef Clock::duration duration;
typedef Clock::time_point time_point;
typedef std::chrono::duration<float, std::milli> milli_duration_f;

class Timer {
  milli_duration_f frameBudget;
  time_point t0;

public:
  Timer();

  void reset();

  float nextSleep();
};