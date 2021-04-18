#pragma once

#include "../config/Config.h"
#include "../logger/Logger.h"
#include <SDL_thread.h>

class DarwinSandbox {
private:
  std::shared_ptr<Logger> logger;
  SDL_Thread *thread;
  SandboxConfig config;

public:
  explicit DarwinSandbox(const std::shared_ptr<Config>& config, const std::shared_ptr<Logger>& logger);
  ~DarwinSandbox();
};

