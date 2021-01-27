#include "Model.h"
#include "Geom.h"

Model::Model(ModelDefinition *definition, Dimensions size, const SpriteAsset *asset)
    : definition(definition),
      size(size),
      collisions(0),
      debounce(new Debounce(100)),
      asset(asset) {}

Model::~Model() {
  delete definition;
}

ModelDefinition* Model::getDefinition() const {
  return definition;
}

const Dimensions &Model::getSize() const {
  return size;
}

void Model::setSize(const Dimensions &size) {
  Model::size = size;
}

const SpriteAsset *Model::getAsset() const {
  return asset;
}

int Model::getCollisions() const {
  return collisions;
}

int Model::recordCollision() {
  if (debounce->shouldCall()) {
    collisions++;
  }
  return collisions;
}
