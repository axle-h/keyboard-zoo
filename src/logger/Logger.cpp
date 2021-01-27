#include "Logger.h"
#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/sinks/dist_sink.h>

namespace spd = spdlog;

Logger::Logger() {
  auto sink = std::make_shared<spd::sinks::dist_sink_st>();
  auto console = std::make_shared<spd::sinks::stdout_color_sink_mt>();
  sink->add_sink(console);
  logger = std::make_shared<spd::logger>("logger", sink);
}

Logger::~Logger() {
  spd::drop("logger");
}
