#pragma once

#include "../assets/Assets.h"
#include "../config/Config.h"
#include "../input/InputState.h"
#include "../models/Sprite.h"
#include "../models/SpriteExplosion.h"
#include <box2d/box2d.h>
#include <set>
#include <utility>
#include <vector>

class World : b2ContactListener {
  std::shared_ptr<Logger> logger;
  std::shared_ptr<Assets> assets;
  WorldConfig config;
  Dimensions worldSize{};
  std::unique_ptr<b2World> world;
  std::set<char> lastKeys;
  std::vector<SpriteExplosion> explosions;

  void buildGroundBody(float x, float y, float width, float height);

  bool tryAddModel(ModelDefinition *definition);

public:
  World(std::shared_ptr<Logger> logger, const std::shared_ptr<Config> &config, std::shared_ptr<Assets> assets);
  ~World() override;

  void BeginContact(b2Contact *contact) override;

  void update(float delta, const InputState *input);

  [[nodiscard]] std::vector<Sprite> getSprites() const;

  [[nodiscard]] std::vector<SpriteExplosion> &getExplosions();

  void setDebugDraw(b2Draw *debugDraw) {
    world->SetDebugDraw(debugDraw);
  }

  void debugDraw() const;
};