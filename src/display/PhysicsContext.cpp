#include "PhysicsContext.h"
#include <SDL2/SDL_timer.h>

PhysicsContext::PhysicsContext(const InputState *input, World *world)
  : input(input), world(world) {
  lock = SDL_CreateSemaphore(1);
  timer = SDL_AddTimer(17, physicsCallback, this);
}

PhysicsContext::~PhysicsContext() {
  SDL_RemoveTimer(timer);
  SDL_DestroySemaphore(lock);
}

uint physicsCallback(uint interval, void *params) {
  auto context = (PhysicsContext *) params;

  auto lock = context->getLock();
  if (SDL_SemWait(lock) >= 0) {
    context->getWorld()->update(interval, context->getInput());
    SDL_SemPost(lock);
  }

  return interval;
}