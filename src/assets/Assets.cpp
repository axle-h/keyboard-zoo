#include "Assets.h"

#include <filesystem>
#include <fstream>
#include <iostream>
#include <nlohmann/json.hpp>

namespace fs = std::filesystem;
using json = nlohmann::json;

Assets::Assets(Config *config) : config(config->getAssets()) {
  auto background = Assets::config.path / "background";
  for (const auto & entry : fs::directory_iterator(Assets::config.path / "background")) {
    backgrounds.push_back(entry.path());
  }

  std::ifstream file(Assets::config.path / "sprites.json");
  std::stringstream buffer;

  json j;
  file >> j;
  file.close();

  for (const auto& it : j.items()) {
    const auto& key = it.key();
    auto value = it.value();

    auto assets = std::vector<SpriteAsset>();
    for (auto& sprite : value) {
      auto polygons = sprite.at("polygons");
      auto position = sprite.at("position");
      auto size = sprite.at("size");
      auto colour = sprite.at("colour");
      auto asset = SpriteAsset {
        .polygons = std::vector<Polygon>(polygons.size()),
        .position = { position.at("x"), position.at("y") },
        .size = { size.at("width"), size.at("height") },
        .colour = { colour.at("r"), colour.at("g"), colour.at("b") },
      };

      for (auto p = 0; p < polygons.size(); p++) {
        auto polygon = polygons.at(p);
        auto vertices = std::vector<b2Vec2>(polygon.size() / 2);

        if (vertices.size() > b2_maxPolygonVertices) {
          throw std::runtime_error("max vertices exceeded");
        }

        for (auto i = 0; i < polygon.size(); i += 2) {
          vertices.at(i / 2).Set(polygon.at(i), polygon.at(i + 1));
        }
        asset.polygons.at(p) = Polygon { vertices };
      }

      assets.push_back(asset);
    }

    characterSprites[key] = assets;
  }
}

const SpriteAsset* Assets::getCharacterSprite(char key) const {
  auto s = std::string(1, key);
  auto sprites = &characterSprites.at(s);
  return sprites->size() == 1
     ? &sprites->at(0)
     : &sprites->at(std::rand() % sprites->size());
}

bool Assets::supportsCharacterSprite(char key) const {
  auto s = std::string(1, key);
  return characterSprites.find(s) != characterSprites.end();
}

std::string Assets::getRandomBackground() const {
  return backgrounds.at(std::rand() % backgrounds.size());
}
