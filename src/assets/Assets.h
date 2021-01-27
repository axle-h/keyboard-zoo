#pragma once

#include "../config/Config.h"
#include "SpriteAsset.h"
#include <map>

class Assets {
  std::vector<std::string> backgrounds;
  std::map<std::string, std::vector<SpriteAsset>> characterSprites;
  AssetConfig config;

public:
  explicit Assets(Config *config);
  [[nodiscard]] const SpriteAsset* getCharacterSprite(char key) const;

  [[nodiscard]] bool supportsCharacterSprite(char key) const;

  [[nodiscard]] std::string getRandomBackground() const;
};
