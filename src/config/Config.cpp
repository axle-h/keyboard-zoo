#include "Config.h"
#include "BuildMeta.h"

#include <SDL.h>
#include <any>
#include <fstream>
#include <iostream>
#include <nlohmann/json.hpp>
#include <sstream>
#include <utility>

namespace fs = std::filesystem;
using json = nlohmann::json;

const auto CONFIG_NAME = "config.json";

class ValidationException : public std::exception {
  std::string message;

  explicit ValidationException(std::string message): message(std::move(message)) {}

public:
  template <class T> static ValidationException create(const std::string& field, const std::string& constraint, T value) {
    std::stringstream ss;
    ss << field << "(" << value << ") must be " << constraint;
    return ValidationException(ss.str());
  }

  [[nodiscard]] const char* what() const noexcept override {
    return message.c_str();
  }
};

Config::Config() {
  title = PROJECT_TITLE;
  auto prefsPath = fs::path(SDL_GetPrefPath("axle-h", PROJECT_NAME));

  filesystem.assets = SDL_GetBasePath();
  filesystem.log = prefsPath / "application.log";
  filesystem.cache = prefsPath / "cache";

  auto configPath = prefsPath / CONFIG_NAME;
  if (fs::exists(configPath) && read(configPath)) {
    validate();
  } else {
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
    auto jRender = j.at("render");

    world = {
      .gravity = jWorld.at("gravity"),
    };
    render = {
      .fullScreen = jRender.at("fullScreen"),
      .debugPhysics = jRender.at("debugPhysics"),
      .pixelsPerMeter = jRender.at("pixelsPerMeter"),
      .internalResolution = {
        jRender.at("internalResolution").at("width"),
        jRender.at("internalResolution").at("height"),
      },
    };
    return true;
  } catch(const std::exception& e) {
    std::cerr << "invalid config " << e.what() << std::endl;
    return false;
  }
}

void Config::write(const std::filesystem::path &path) {
  std::ofstream file(path, std::ios_base::trunc);

  json j = {
    {"world", {{"gravity", world.gravity}}},
    { "render", {
      { "fullScreen", render.fullScreen },
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
  render.fullScreen = true;
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
  between("render.pixelsPerMeter", render.pixelsPerMeter, 1.f, 50.f);
  between("render.internalResolution.width", render.internalResolution.width, 640, 3840);
  between("render.internalResolution.height", render.internalResolution.height, 480, 2160);
}
