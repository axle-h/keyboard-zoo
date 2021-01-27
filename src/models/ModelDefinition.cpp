#include "ModelDefinition.h"

ModelDefinition::ModelDefinition(ModelType type) : type(type) {}
ModelType ModelDefinition::getType() const {
  return type;
}