#pragma once

enum ModelType {
  ModelType_Ground,
  ModelType_Character,
};

class ModelDefinition {
  ModelType type;

public:
  explicit ModelDefinition(ModelType type);
  [[nodiscard]] ModelType getType() const;
};