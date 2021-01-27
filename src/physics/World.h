#pragma once

#include "../assets/Assets.h"
#include "../config/Config.h"
#include "../input/InputState.h"
#include "../models/Model.h"
#include "../models/ModelDefinition.h"
#include <box2d/box2d.h>
#include <vector>

class Sprite {
public:
  Model *model;
  float angle;
  Point position;
  Point center;

  Sprite(Model *model, float angle, const Point &position, const Point &center)
      : model(model), angle(angle), position(position), center(center) {}
};

class Particle {
public:
  std::vector<b2Vec2> vertices;
  b2Transform transform;
  b2Vec2 velocity;
  float angularVelocity;

  [[nodiscard]] std::vector<b2Vec2> getVertices() const {
    auto result = vertices;
    for (auto& v : result) {
      v = b2Mul(transform, v);
    }
    return result;
  }
};

class DestroyedSprite : public Sprite {
public:
  int framesRemaining;
  float percentRemaining;
  std::vector<Particle> particles;

  DestroyedSprite(Sprite *sprite, int framesRemaining)
      : Sprite(sprite->model, sprite->angle, sprite->position, sprite->center),
        framesRemaining(framesRemaining),
        percentRemaining(1.f) {}
};

class ContactListener : public b2ContactListener {
  b2World *world;

public:
  explicit ContactListener(b2World *world) : world(world) {}
  void BeginContact(b2Contact *contact) override;
};

class World {
  Assets *assets;
  WorldConfig config;
  Dimensions worldSize;
  ContactListener *listener;
  b2World *world;
  std::vector<char> lastKeys;
  std::vector<DestroyedSprite> destroyedSprites;

  void buildGroundBody(float x, float y, float width, float height);

  bool tryAddModel(ModelDefinition *definition);

public:
  World(Config *config, Assets *assets);
  ~World();

  void update(float delta, const InputState *input);

  [[nodiscard]] std::vector<Sprite> getSprites() const;

  [[nodiscard]] std::vector<DestroyedSprite> getDestroyedSprites() const;


  void setDebugDraw(b2Draw *debugDraw) {
    world->SetDebugDraw(debugDraw);
  }

  void debugDraw() const;
};