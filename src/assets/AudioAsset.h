#pragma once

#include <string>
#include <utility>

enum class AudioAssetType {
  Create,
  Destroy,
  Music,
};

class AudioAsset {
  std::string name;
  AudioAssetType type;
  std::string path;

public:
  AudioAsset(std::string name, AudioAssetType type, std::string path)
    : name(std::move(name)), type(type), path(std::move(path)) {}

  [[nodiscard]] inline const std::string &getName() const {
    return name;
  }

  [[nodiscard]] inline AudioAssetType getType() const {
    return type;
  }

  [[nodiscard]] inline const std::string &getPath() const {
    return path;
  }
};
