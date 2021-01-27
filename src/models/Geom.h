#pragma once

struct Point {
  float x;
  float y;
};

struct Dimensions {
  float width;
  float height;
};

inline Dimensions operator*(Dimensions a, float scalar) {
  return Dimensions{.width = a.width * scalar, .height = a.height * scalar};
}