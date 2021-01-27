#include "assets/Assets.h"
#include "config/Config.h"
#include "display/Display.h"
#include "logger/Logger.h"
#include "timer/Timer.h"


int main() {
  auto logger = std::make_unique<Logger>();

  try {
    auto config = std::make_unique<Config>(logger.get());
    auto timer =  std::make_unique<Timer>();
    auto assets = std::make_unique<Assets>(config.get());
    auto world = std::make_unique<World>(config.get(), assets.get());
    auto display = std::make_unique<Display>(logger.get(), config.get(), assets.get(), world.get());

    while (display->next()) {
      timer->nextSleep();
    }

    return 0;
  } catch(std::exception& e) {
    logger->critical(e.what());
  }

}
