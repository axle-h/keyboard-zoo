#include "Debounce.h"

Debounce::Debounce(const int &debounceForMillis)
: debounceFor(std::chrono::milliseconds(debounceForMillis)), lastCall() {}

bool Debounce::shouldCall() {
  auto t1 = Clock::now();
  auto result = (t1 - lastCall) > debounceFor;
  if (result) {
    lastCall = t1;
    return true;
  }
  return false;
}
