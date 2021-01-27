#pragma once

#include "../config/Config.h"
#include <SDL_render.h>
#include <box2d/box2d.h>
#include <vector>

struct Vertices {
  std::vector<Sint16> vx;
  std::vector<Sint16> vy;
};

class DebugDrawDisplayAdapter : public b2Draw {
  SDL_Renderer *renderer;
  RenderConfig config;

  [[nodiscard]] Sint16 X(float x) const;
  [[nodiscard]] Sint16 Y(float y) const;
  Vertices getVertices(const b2Vec2 *vertices, int32 vertexCount) const;

public:
  explicit DebugDrawDisplayAdapter(SDL_Renderer *renderer, RenderConfig config);

  void DrawPolygon(const b2Vec2 *vertices, int32 vertexCount, const b2Color &color) override;

  void DrawSolidPolygon(const b2Vec2 *vertices, int32 vertexCount, const b2Color &color) override;

  void DrawCircle(const b2Vec2 &center, float radius, const b2Color &color) override;

  void DrawSolidCircle(const b2Vec2 &center, float radius, const b2Vec2 &axis, const b2Color &color) override;

  void DrawSegment(const b2Vec2 &p1, const b2Vec2 &p2, const b2Color &color) override;

  void DrawTransform(const b2Transform &xf) override;

  void DrawPoint(const b2Vec2 &p, float size, const b2Color &color) override;
};
