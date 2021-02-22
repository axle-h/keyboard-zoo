#include "PhysicsContext.h"
#include <SDL.h>

PhysicsContext::PhysicsContext(InputState *input, World *world)
  : input(input), world(world) {
  lock = SDL_CreateSemaphore(1);
  timer = SDL_AddTimer(17, physicsCallback, this);
}

PhysicsContext::~PhysicsContext() {
  SDL_RemoveTimer(timer);
  SDL_DestroySemaphore(lock);
}

uint32 physicsCallback(uint32 interval, void *params) {
  auto context = (PhysicsContext *) params;

  auto lock = context->getLock();
  if (SDL_SemWait(lock) >= 0) {
    context->getWorld()->update(interval, context->getInput());
    SDL_SemPost(lock);
  }

  return interval;
}