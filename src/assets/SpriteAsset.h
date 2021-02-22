#pragma once

#include "../models/Geom.h"
#include <box2d/box2d.h>
#include <utility>
#include <vector>
#include <string>

struct Colour {
  int r;
  int g;
  int b;
};

struct Polygon {
  std::vector<b2Vec2> vertices;
};

class SpriteAsset {
  std::string name;
  std::vector<Polygon> polygons;
  Point position;
  Dimensions size;
  Colour colour;

public:
  SpriteAsset(std::string name,
              std::vector<Polygon> polygons,
              const Point &position,
              const Dimensions &size,
              const Colour &colour)
      : name(std::move(name)), polygons(std::move(polygons)), position(position), size(size), colour(colour) {}

  [[nodiscard]] const std::string &getName() const {
    return name;
  }

  [[nodiscard]] const std::vector<Polygon> &getPolygons() const {
    return polygons;
  }

  [[nodiscard]] const Point &getPosition() const {
    return position;
  }

  [[nodiscard]] const Dimensions &getSize() const {
    return size;
  }

  [[nodiscard]] const Colour &getColour() const {
    return colour;
  }
};
