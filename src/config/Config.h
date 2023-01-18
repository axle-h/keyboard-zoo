#pragma once

#include <filesystem>

struct WorldConfig {
  float gravity;
};

struct FilesystemConfig {
  std::filesystem::path assets;
  std::filesystem::path log;
};

struct SandboxConfig {
  bool enabled;
};

struct Resolution {
  int width;
  int height;
};

struct RenderConfig {
  bool fullScreen;
  bool debugPhysics;
  float pixelsPerMeter;
  Resolution internalResolution;
};

class Config {
  SandboxConfig sandbox{};
  WorldConfig world{};
  FilesystemConfig filesystem{};
  RenderConfig render{};
  std::string title;

  bool read(const std::filesystem::path &path);
  void write(const std::filesystem::path &path);
  void defaults();
  void validate() const;

public:
  explicit Config();

  [[nodiscard]] const SandboxConfig &getSandbox() const {
    return sandbox;
  }

  [[nodiscard]] inline const WorldConfig &getWorld() const {
    return world;
  }

  [[nodiscard]] inline const FilesystemConfig &getFilesystem() const {
    return filesystem;
  }

  [[nodiscard]] inline const RenderConfig &getRender() const {
    return render;
  }

  [[nodiscard]] const std::string &getTitle() const {
    return title;
  }
};
