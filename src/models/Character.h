#pragma once

#include "Model.h"
#include "ModelDefinition.h"

class Character : public ModelDefinition {
  char value;
public:
  explicit Character(char value);

  [[nodiscard]] char getValue() const;

  void setValue(char value);
};
