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
  std::string compressedPath;
  std::string decompressedPath;

public:
  AudioAsset(std::string name, AudioAssetType type, std::string compressedPath, std::string decompressedPath)
    : name(std::move(name)), type(type), compressedPath(std::move(compressedPath)), decompressedPath(std::move(decompressedPath)) {}

  [[nodiscard]] inline const std::string &getName() const {
    return name;
  }

  [[nodiscard]] inline AudioAssetType getType() const {
    return type;
  }

  [[nodiscard]] inline const std::string &getCompressedPath() const {
    return compressedPath;
  }

  [[nodiscard]] inline const std::string &getDecompressedPath() const {
    return decompressedPath;
  }

  inline void setDecompressedPath(const std::string &decompressedPath) {
    AudioAsset::decompressedPath = decompressedPath;
  }

  inline void merge(AudioAsset *other) {
    if (compressedPath.empty()) {
      compressedPath = other->compressedPath;
    }
    if (decompressedPath.empty()) {
      decompressedPath = other->decompressedPath;
    }
  }
};
