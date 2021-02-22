#pragma once

#include <vector>
#include <box2d/box2d.h>
#include "Model.h"

const int DESTROYED_SPRITE_FRAMES = 60 * 1.5;

class Particle {
  std::vector<b2Vec2> vertices;
  b2Transform transform;
  b2Vec2 velocity;
  float angularVelocity;

public:
  Particle(const std::vector<b2Vec2> &vertices, const b2Transform &transform, const b2Vec2 &velocity, float angularVelocity)
      : vertices(vertices), transform(transform), velocity(velocity), angularVelocity(angularVelocity) {}

  [[nodiscard]] std::vector<b2Vec2> getVertices() const {
    auto result = vertices;
    for (auto& v : result) {
      v = b2Mul(transform, v);
    }
    return result;
  }

  void nextFrame(float timeStep) {
    transform.p.x += velocity.x * timeStep;
    transform.p.y += velocity.y * timeStep;
    transform.q.Set(transform.q.GetAngle() + angularVelocity);
  }
};

class SpriteExplosion {
  std::vector<Particle> particles;
  const SpriteAsset *asset;
  bool destroyed = false;
  int frames = 0;

public:
  SpriteExplosion(Model *model, std::vector<Particle> particles)
    : asset(model->getAsset()), particles(std::move(particles)) {}

  [[nodiscard]] const std::vector<Particle> &getParticles() const {
    return particles;
  }

  [[nodiscard]] const inline SpriteAsset *getAsset() const {
    return asset;
  }

  [[nodiscard]] bool inline isDestroyed() const {
    return destroyed;
  }

  void inline setDestroyed() {
    SpriteExplosion::destroyed = true;
  }

  bool nextFrame(float timeStep) {
    if (++frames >= DESTROYED_SPRITE_FRAMES) {
      return true;
    }
    for (auto& particle : particles) {
      particle.nextFrame(timeStep);
    }
    return false;
  }

  [[nodiscard]] float getPercent() const {
    return (float) frames / DESTROYED_SPRITE_FRAMES;
  }
};