#pragma once

#include "Model.h"
#include "ModelDefinition.h"

class Ground : public ModelDefinition {
public:
  explicit Ground() : ModelDefinition(ModelType_Ground) {}
};
