#include "Config.h"

#include <any>
#include <fstream>
#include <nlohmann/json.hpp>
#include <platform_folders.h>
#include <utility>

namespace fs = std::filesystem;
using json = nlohmann::json;

const auto CONFIG_NAME = "config.json";

fs::path getOptionsPath() {
  // TODO get project name from cmake
  return fs::path(sago::getConfigHome()) / "baby-smash";
}

class ValidationException : public std::exception {
  std::string message;

  explicit ValidationException(std::string message): message(std::move(message)) {}

public:
  template <class T> static ValidationException create(const std::string& field, const std::string& constraint, T value) {
    std::stringstream ss;
    ss << field << "(" << value << ") must be " << constraint;
    return ValidationException(ss.str());
  }

  [[nodiscard]] const char *what() const _GLIBCXX_TXN_SAFE_DYN _GLIBCXX_NOTHROW override {
    return message.c_str();
  }
};

Config::Config(Logger *logger) : logger(logger) {
  auto configDirectory = getOptionsPath();
  if (!fs::exists(configDirectory) && !fs::create_directories(configDirectory)) {
    logger->error("cannot create config directory {}", configDirectory.string());
    throw std::exception();
  }

  auto configPath = configDirectory / CONFIG_NAME;
  if (fs::exists(configPath) && read(configPath)) {
    validate();
  } else {
    logger->info("writing default configuration to {}", configPath.string());
    defaults();
    write(configPath);
  }
}

bool Config::read(const std::filesystem::path &path) {
  try {
    std::ifstream file(path);
    json j;
    file >> j;

    auto jWorld = j.at("world");
    auto jAssets = j.at("assets");
    auto jRender = j.at("render");

    world = {
      .gravity = jWorld.at("gravity"),
    };
    assets = {
      .path = fs::path(jAssets.at("path")),
    };
    render = {
      .debugPhysics = jRender.at("debugPhysics"),
      .pixelsPerMeter = jRender.at("pixelsPerMeter"),
      .internalResolution = {
        jRender.at("internalResolution").at("width"),
        jRender.at("internalResolution").at("height"),
      },
    };
    return true;
  } catch(const std::exception& e) {
    logger->error("invalid config {}", e.what());
    return false;
  }
}

void Config::write(const std::filesystem::path &path) {
  std::ofstream file(path, std::ios_base::trunc);

  json j = {
    {"world", {{"gravity", world.gravity}}},
    {"assets", {{"path", assets.path}}},
    { "render", {
      { "debugPhysics", render.debugPhysics },
      { "pixelsPerMeter", render.pixelsPerMeter },
      { "internalResolution", {
        { "width", render.internalResolution.width },
        { "height", render.internalResolution.height }
      }}
    }}
  };

  file << std::setw(4) << j << std::endl;
  file.close();
}

void Config::defaults() {
  world.gravity = -1.f;
  assets.path = fs::path("..") / "assets";
  render.debugPhysics = false;
  render.pixelsPerMeter = 20.f;
  render.internalResolution = { 1920, 1080 };
}

template <class T> void between(const std::string& field, T value, T min, T max) {
  if (value < min || value > max) {
    std::stringstream ss;
    ss << "between " << min << " and " << max;
    throw ValidationException::create(field, ss.str(), value);
  }
}

void Config::validate() const {
  between("world.gravity", world.gravity, -20.f, 20.f);

  if (fs::status(assets.path).type() != fs::file_type::directory) {
    throw ValidationException::create("assets.path", "must be a directory", assets.path);
  }

  between("render.pixelsPerMeter", render.pixelsPerMeter, 1.f, 50.f);
  between("render.internalResolution.width", render.internalResolution.width, 640, 3840);
  between("render.internalResolution.height", render.internalResolution.height, 480, 2160);
}
