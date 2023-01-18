#pragma once

#include "../config/Config.h"
#include "../logger/Logger.h"
#include "AudioAsset.h"
#include "SpriteAsset.h"
#include <map>

class Assets {
  const std::vector<std::string> VIDEO_EXTENSIONS = std::vector<std::string>{".mov", ".mp4"};
  const std::vector<std::string> AUDIO_EXTENSIONS = std::vector<std::string>{".ogg"};
  std::shared_ptr<Logger> logger;
  std::vector<std::string> videos;
  std::map<std::string, std::vector<SpriteAsset>> sprites;
  FilesystemConfig config;

public:
  Assets(const std::shared_ptr<Config>& config, std::shared_ptr<Logger> logger);

  [[nodiscard]] const SpriteAsset *getSprite(char key) const;

  [[nodiscard]] bool supportsSprite(char key) const;

  [[nodiscard]] std::string getRandomVideo() const;

  [[nodiscard]] std::vector<AudioAsset> getAudioAssets() const;
};