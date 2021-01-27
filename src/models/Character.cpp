#include "Character.h"
#include "ModelDefinition.h"

Character::Character(char value)
    : ModelDefinition(ModelType_Character), value(value) {
}

char Character::getValue() const {
  return value;
}

void Character::setValue(char value) {
  Character::value = value;
}
