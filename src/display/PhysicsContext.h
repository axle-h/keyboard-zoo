#pragma once

#include <SDL.h>
#include "../input/InputState.h"
#include "../physics/World.h"

uint32 physicsCallback(uint32 interval, void *params);

class PhysicsContext {
  SDL_semaphore *lock;
  World *world;
  InputState *input;
  int timer;

public:
  PhysicsContext(InputState *input, World *world);
  ~PhysicsContext();

  [[nodiscard]] const InputState *getInput() const {
    return input;
  }

  [[nodiscard]] World *getWorld() const {
    return world;
  }

  [[nodiscard]] SDL_semaphore *getLock() const {
    return lock;
  }
};

