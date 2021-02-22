#pragma once

#include <memory>

#include "../assets/SpriteAsset.h"
#include "../timer/Debounce.h"
#include "Geom.h"
#include "ModelDefinition.h"

class Model {
  Dimensions size;
  ModelDefinition* definition;
  const SpriteAsset *asset;
  int collisions;
  std::unique_ptr<Debounce> debounce;
  bool created = false;

public:
  Model(ModelDefinition *definition, Dimensions size, const SpriteAsset *asset);
  ~Model();

  [[nodiscard]] const inline Dimensions &getSize() const {
    return size;
  }

  void inline setSize(const Dimensions &size) {
    Model::size = size;
  }

  [[nodiscard]] inline ModelDefinition* getDefinition() const {
    return definition;
  }

  [[nodiscard]] const inline SpriteAsset *getAsset() const {
    return asset;
  }

  [[nodiscard]] int inline getCollisions() const {
    return collisions;
  }

  [[nodiscard]] bool inline isCreated() const {
    return created;
  }

  void inline setCreated() {
    Model::created = true;
  }

  int recordCollision();
};
