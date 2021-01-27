#include "DebugDrawDisplayAdapter.h"

#include <SDL2_gfxPrimitives.h>

DebugDrawDisplayAdapter::DebugDrawDisplayAdapter(SDL_Renderer *renderer, RenderConfig config)
    : renderer(renderer), config(config) {
  AppendFlags(e_shapeBit);
}

Sint16 DebugDrawDisplayAdapter::X(float x) const {
  return std::round(x * config.pixelsPerMeter);
}

Sint16 DebugDrawDisplayAdapter::Y(float y) const {
  return (float) config.internalResolution.height - std::round(y * config.pixelsPerMeter);
}

Vertices DebugDrawDisplayAdapter::getVertices(const b2Vec2 *vertices, int32 vertexCount) const {
  auto result = Vertices{ .vx = std::vector<Sint16>(vertexCount), .vy = std::vector<Sint16>(vertexCount) };

  for (auto i = 0; i < vertexCount; i++) {
    auto vertex = vertices[i];
    result.vx.at(i) = X(vertex.x);
    result.vy.at(i) = Y(vertex.y);
  }

  return result;
}

void DebugDrawDisplayAdapter::DrawPolygon(const b2Vec2 *vertices, int32 vertexCount, const b2Color &color) {
  auto vs = getVertices(vertices, vertexCount);

  // TODO maybe use this https://github.com/rtrussell/BBCSDL/blob/43f045f16e77229bc4c5cddfee212b908af48702/src/SDL2_gfxPrimitives.c#L4815
  polygonRGBA(renderer, vs.vx.data(), vs.vy.data(), vertexCount, 255 * color.r, 255 * color.g, 255 * color.b, 255 * color.a);
}

void DebugDrawDisplayAdapter::DrawSolidPolygon(const b2Vec2 *vertices, int32 vertexCount, const b2Color &color) {
  auto vs = getVertices(vertices, vertexCount);

  // TODO maybe use this https://github.com/rtrussell/BBCSDL/blob/43f045f16e77229bc4c5cddfee212b908af48702/src/SDL2_gfxPrimitives.c#L4815
  filledPolygonRGBA(renderer, vs.vx.data(), vs.vy.data(), vertexCount, 255 * color.r, 255 * color.g, 255 * color.b, 255 * color.a);
}

void DebugDrawDisplayAdapter::DrawCircle(const b2Vec2 &center, float radius, const b2Color &color) {
  circleRGBA(renderer, X(center.x), Y(center.y), X(radius),
             255 * color.r, 255 * color.g, 255 * color.b, 255 * color.a);
}

void DebugDrawDisplayAdapter::DrawSolidCircle(const b2Vec2 &center, float radius, const b2Vec2 &axis, const b2Color &color) {
  filledCircleRGBA(renderer, X(center.x), Y(center.y), X(radius),
                   255 * color.r, 255 * color.g, 255 * color.b, 255 * color.a);
}

void DebugDrawDisplayAdapter::DrawSegment(const b2Vec2 &p1, const b2Vec2 &p2, const b2Color &color) {}

void DebugDrawDisplayAdapter::DrawTransform(const b2Transform &xf) {}

void DebugDrawDisplayAdapter::DrawPoint(const b2Vec2 &p, float size, const b2Color &color) {}
