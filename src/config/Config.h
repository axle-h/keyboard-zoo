#pragma once

#include "../logger/Logger.h"
#include <filesystem>

struct WorldConfig {
  float gravity;
};

struct AssetConfig {
  std::filesystem::path path;
};

struct Resolution {
  int width;
  int height;
};

struct RenderConfig {
  bool debugPhysics;
  float pixelsPerMeter;
  Resolution internalResolution;
};

class Config {
  Logger *logger;
  WorldConfig world{};
  AssetConfig assets{};
  RenderConfig render{};

  bool read(const std::filesystem::path &path);
  void write(const std::filesystem::path &path);
  void defaults();
  void validate() const;

public:
  explicit Config(Logger *logger);

  [[nodiscard]] inline const WorldConfig &getWorld() const {
    return world;
  }

  [[nodiscard]] inline const AssetConfig &getAssets() const {
    return assets;
  }

  [[nodiscard]] inline const RenderConfig &getRender() const {
    return render;
  }
};
