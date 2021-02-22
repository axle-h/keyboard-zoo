#include "Assets.h"

#include <filesystem>
#include <fstream>
#include <iostream>
#include <nlohmann/json.hpp>
#include <sstream>
#include <utility>

namespace fs = std::filesystem;
using json = nlohmann::json;

Assets::Assets(const std::shared_ptr<Config>& config, std::shared_ptr<Logger> logger) : config(config->getFilesystem()), logger(std::move(logger)) {
  auto assetsPath = Assets::config.assets;
  for (const auto &entry : fs::directory_iterator(assetsPath / "video")) {
    auto extension = entry.path().extension();
    if (std::find(VIDEO_EXTENSIONS.begin(), VIDEO_EXTENSIONS.end(), extension) == VIDEO_EXTENSIONS.end()) {
      continue;
    }
    videos.push_back(entry.path());
  }

  std::ifstream file(assetsPath / "sprites.json");
  std::stringstream buffer;

  json j;
  file >> j;
  file.close();

  for (const auto &it : j.items()) {
    const auto &key = it.key();
    auto value = it.value();

    auto assets = std::vector<SpriteAsset>();
    for (auto &sprite : value) {
      auto polygons = sprite.at("polygons");
      auto position = sprite.at("position");
      auto size = sprite.at("size");
      auto colour = sprite.at("colour");
      std::string name = sprite.at("name");

      auto result = std::vector<Polygon>(polygons.size());
      for (auto p = 0; p < polygons.size(); p++) {
        auto polygon = polygons.at(p);
        auto vertices = std::vector<b2Vec2>(polygon.size() / 2);

        if (vertices.size() > b2_maxPolygonVertices) {
          throw std::runtime_error("max vertices exceeded");
        }

        for (auto i = 0; i < polygon.size(); i += 2) {
          vertices.at(i / 2).Set(polygon.at(i), polygon.at(i + 1));
        }
        result.at(p) = Polygon{vertices};
      }

      auto asset = SpriteAsset(name, result,
                               Point{position.at("x"), position.at("y")},
                               Dimensions{size.at("width"), size.at("height")},
                               Colour{colour.at("r"), colour.at("g"), colour.at("b")});
      assets.push_back(asset);
    }

    sprites[key] = assets;
  }
}

const SpriteAsset *Assets::getSprite(char key) const {
  auto s = std::string(1, key);
  auto result = &sprites.at(s);
  return result->size() == 1
           ? &result->at(0)
           : &result->at(std::rand() % result->size());
}

bool Assets::supportsSprite(char key) const {
  auto s = std::string(1, key);
  return sprites.find(s) != sprites.end();
}

std::string Assets::getRandomVideo() const {
  return videos.at(std::rand() % videos.size());
}

std::string getFolder(AudioAssetType type) {
  switch (type) {
    case AudioAssetType::Create:
      return "create";
    case AudioAssetType::Destroy:
      return "destroy";
    case AudioAssetType::Music:
      return "music";
  }
}

std::vector<AudioAsset> Assets::getAudioAssets() const {
  auto map = std::unordered_map<std::string, AudioAsset>();

  for (auto type : {AudioAssetType::Create, AudioAssetType::Destroy, AudioAssetType::Music}) {
    auto audioPath = config.assets / "audio" / getFolder(type);
    for (const auto &file : fs::directory_iterator(audioPath)) {
      const auto &filePath = file.path();
      auto extension = file.path().extension();

      if (std::find(AUDIO_EXTENSIONS.begin(), AUDIO_EXTENSIONS.end(), extension) == AUDIO_EXTENSIONS.end()) {
        continue;
      }

      auto isWav = extension == ".wav";
      auto name = filePath.filename().replace_extension().string();
      std::stringstream ss;
      ss << getFolder(type) << "-" << name;
      auto key = ss.str();

      AudioAsset asset(name, type, isWav ? "" : filePath.string(), isWav ? filePath.string() : "");
      auto it = map.find(key);
      if (it == map.end()) {
        map.emplace(std::make_pair(key, asset));
      } else {
        it->second.merge(&asset);
      }
    }
  }

  auto result = std::vector<AudioAsset>();
  for (const auto &pair : map) {
    auto asset = pair.second;
    if (asset.getDecompressedPath().empty()) {
      auto path = config.cache / getFolder(asset.getType());
      if (!fs::exists(path) && !fs::create_directories(path)) {
        logger->error("cannot create config directory {}", path.string());
        throw std::exception();
      }

      auto filePath = path / fs::path(asset.getCompressedPath()).filename().replace_extension(".wav");
      asset.setDecompressedPath(filePath);
    }
    result.push_back(asset);
  }

  return result;
}