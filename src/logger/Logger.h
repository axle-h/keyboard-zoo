#pragma once

#include "spdlog/spdlog.h"

namespace spd = spdlog;

class Logger {
  std::shared_ptr<spd::logger> logger;

public:
  explicit Logger(const std::string& path);
  ~Logger();

  template<typename T>
  inline void debug(const T &msg) {
    logger->debug(msg);
  }

  template<typename FormatString, typename... Args>
  inline void debug(const FormatString &fmt, Args&&...args) {
    logger->debug(fmt, std::forward<Args>(args)...);
  }

  template<typename T>
  inline void info(const T &msg) {
    logger->info(msg);
  }

  template<typename FormatString, typename... Args>
  inline void info(const FormatString &fmt, Args&&...args) {
    logger->info(fmt, std::forward<Args>(args)...);
  }

  template<typename T>
  inline void warn(const T &msg) {
    logger->warn(msg);
  }

  template<typename FormatString, typename... Args>
  inline void warn(const FormatString &fmt, Args&&...args) {
    logger->warn(fmt, std::forward<Args>(args)...);
  }

  template<typename T>
  inline void error(const T &msg) {
    logger->error(msg);
  }

  template<typename FormatString, typename... Args>
  inline void error(const FormatString &fmt, Args&&...args) {
    logger->error(fmt, std::forward<Args>(args)...);
  }

  template<typename T>
  inline void critical(const T &msg) {
    logger->critical(msg);
  }

  template<typename FormatString, typename... Args>
  inline void critical(const FormatString &fmt, Args&&...args) {
    logger->critical(fmt, std::forward<Args>(args)...);
  }
};
