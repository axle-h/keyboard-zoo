#pragma once

#include <SDL2/SDL_thread.h>
#include "../input/InputState.h"
#include "../physics/World.h"

uint physicsCallback(uint interval, void *params);

class PhysicsContext {
  SDL_semaphore *lock;
  World *world;
  const InputState *input;
  int timer;

public:
  PhysicsContext(const InputState *input, World *world);
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

