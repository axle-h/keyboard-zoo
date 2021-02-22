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

int Model::recordCollision() {
  if (debounce->shouldCall()) {
    collisions++;
  }
  return collisions;
}
