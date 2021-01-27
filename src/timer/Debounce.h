#pragma once

#include <chrono>

typedef std::chrono::steady_clock Clock;
typedef std::chrono::duration<long, std::milli> milli_duration;
typedef Clock::time_point time_point;

class Debounce {
  time_point lastCall;
  milli_duration debounceFor;

public:
  explicit Debounce(const int &debounceForMillis);

  bool shouldCall();
};
