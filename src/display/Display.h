#pragma once

#include "../input/InputState.h"
#include "../physics/World.h"
#include "../video/BackgroundContext.h"
#include "../video/FrameScaler.h"
#include "../video/FrameService.h"
#include "DebugDrawDisplayAdapter.h"
#include "PhysicsContext.h"
#include <SDL2/SDL.h>
#include <box2d/box2d.h>

class Display {
  Logger *logger;
  Assets *assets;
  World *world;
  InputState input;
  RenderConfig config;
  SDL_Window *window;
  SDL_Event event{};
  SDL_Renderer *renderer;
  SDL_Texture *target;
  SDL_Texture *spriteSheet;
  SDL_Texture *backgroundTexture;
  std::unique_ptr<BackgroundContext> background;
  int backgroundTimerId;
  std::unique_ptr<DebugDrawDisplayAdapter> debugDraw;
  std::unique_ptr<PhysicsContext> physics;

  void drawBackground();
  void drawWorld();

  [[nodiscard]] inline float X(float x) const {
    return x * config.pixelsPerMeter;
  }

  [[nodiscard]] inline float Y(float y) const {
    return (float) config.internalResolution.height - y * config.pixelsPerMeter;
  }

  [[nodiscard]] inline float D(float d) const {
    return d * config.pixelsPerMeter;
  }

public:
  Display(Logger *logger, Config *config, Assets *assets, World *world);
  ~Display();

  bool next();

};
