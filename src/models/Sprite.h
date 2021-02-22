#pragma once

#include "Model.h"

class Sprite {
public:
  Model *model;
  float angle;
  Point position;
  Point center;

  Sprite(Model *model, float angle, const Point &position, const Point &center)
    : model(model), angle(angle), position(position), center(center) {}
};