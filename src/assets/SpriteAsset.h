#pragma once

#include "../models/Geom.h"
#include <box2d/box2d.h>
#include <vector>

struct Colour {
  int r;
  int g;
  int b;
};

struct Polygon {
  std::vector<b2Vec2> vertices;
};

struct SpriteAsset {
  std::vector<Polygon> polygons;
  Point position;
  Dimensions size;
  Colour colour;
};
