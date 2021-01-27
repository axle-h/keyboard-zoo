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

public:
  Model(ModelDefinition *definition, Dimensions size, const SpriteAsset *asset);
  ~Model();

  [[nodiscard]] const Dimensions &getSize() const;

  void setSize(const Dimensions &size);

  [[nodiscard]] ModelDefinition* getDefinition() const;

  [[nodiscard]] const SpriteAsset *getAsset() const;

  [[nodiscard]] int getCollisions() const;

  int recordCollision();
};
