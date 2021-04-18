#include "assets/Assets.h"
#include "display/Display.h"
#include "timer/Timer.h"

#if defined(__APPLE__)
  #include "sandbox/DarwinSandbox.h"
#endif

int main(int argv, char** args) {
  auto config = std::make_shared<Config>();
  auto logger = std::make_shared<Logger>(config->getFilesystem().log.string());

#if defined(__APPLE__)
  auto sandbox = std::make_unique<DarwinSandbox>(config, logger);
#endif

  try {
    auto timer = std::make_unique<Timer>();
    auto assets = std::make_shared<Assets>(config, logger);
    auto audio = std::make_shared<AudioService>(assets, logger);
    auto world = std::make_shared<World>(logger, config, assets);
    auto display = std::make_shared<Display>(logger, config, assets, world, audio);

    while (display->next()) {
      timer->nextSleep();
    }

    return 0;
  } catch(std::exception& e) {
    logger->critical(e.what());
    return 1;
  }
}
