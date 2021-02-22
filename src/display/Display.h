#pragma once

#include "../input/InputState.h"
#include "../media/AudioService.h"
#include "../media/FrameService.h"
#include "../media/VideoContext.h"
#include "../media/VideoScaler.h"
#include "../physics/World.h"
#include "DebugDrawDisplayAdapter.h"
#include "PhysicsContext.h"
#include <SDL.h>
#include <box2d/box2d.h>

class Display {
  std::shared_ptr<Logger> logger;
  std::shared_ptr<Assets> assets;
  std::shared_ptr<World> world;
  std::shared_ptr<AudioService> audio;
  std::shared_ptr<InputState> input;
  RenderConfig config;
  SDL_Window *window;
  SDL_Event event{};
  SDL_Renderer *renderer;
  SDL_Texture *target;
  SDL_Texture *spriteSheet;
  SDL_Texture *backgroundTexture;
  std::unique_ptr<VideoContext> background;
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
  Display(std::shared_ptr<Logger> logger, const std::shared_ptr<Config> &config,
          const std::shared_ptr<Assets>& assets, const std::shared_ptr<World>& world, const std::shared_ptr<AudioService> &audio);
  ~Display();

  bool next();
};
