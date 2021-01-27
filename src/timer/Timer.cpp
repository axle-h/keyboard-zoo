#include <cmath>
#include <thread>

#include "Timer.h"

Timer::Timer() : frameBudget() {
  auto frameMicros = (long) std::round(1000000 / 60.0);
  frameBudget = std::chrono::duration_cast<duration>(std::chrono::microseconds(frameMicros));
  t0 = Clock::now();
}

void Timer::reset() {
  t0 = Clock::now();
}

float Timer::nextSleep() {
  auto t1 = Clock::now();
  milli_duration_f delta = t1 - t0;
  t0 = t1;
  auto frame_delta = frameBudget - delta;
  if (frame_delta.count() > 0) {
    std::this_thread::sleep_for(frame_delta);
  }

  return std::max(delta.count(), frameBudget.count());
}
