#include "Logger.h"
#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/sinks/basic_file_sink.h>
#include <spdlog/sinks/dist_sink.h>

namespace spd = spdlog;

Logger::Logger(const std::string& path) {
  auto sink = std::make_shared<spd::sinks::dist_sink_st>();
  auto console = std::make_shared<spd::sinks::stdout_color_sink_mt>();
  auto file = std::make_shared<spdlog::sinks::basic_file_sink_mt>(path, true);
  sink->add_sink(console);
  sink->add_sink(file);
  logger = std::make_shared<spd::logger>("logger", sink);
}

Logger::~Logger() {
  spd::drop("logger");
}
