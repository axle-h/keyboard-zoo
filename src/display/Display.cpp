#include <algorithm>
#include <stdexcept>
#include <SDL2/SDL_image.h>
#include <SDL2_gfxPrimitives.h>
#include "DebugDrawDisplayAdapter.h"
#include "Display.h"

Display::Display(Logger *logger, Config *config, Assets *assets, World *world)
    : assets(assets), logger(logger), config(config->getRender()), world(world) {
  input = InputState{
    .quit = false,
    .up = false,
    .down = false,
    .left = false,
    .right = false,
    .keys = std::vector<char>(),
  };

  if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_TIMER) < 0) {
    throw std::runtime_error("Cannot initialise SDL");
  }

  auto resolution = Display::config.internalResolution;

  window = SDL_CreateWindow("baby-smash",
                            SDL_WINDOWPOS_CENTERED,
                            SDL_WINDOWPOS_CENTERED,
                            resolution.width / 2, resolution.height / 2,
                            SDL_WINDOW_SHOWN | SDL_WINDOW_RESIZABLE);

  if (!window) {
    throw std::runtime_error("Cannot create SDL window");
  }

  renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED | SDL_RENDERER_TARGETTEXTURE);

  if (!renderer) {
    throw std::runtime_error("Cannot create SDL renderer");
  }

  SDL_RenderSetLogicalSize(renderer, resolution.width, resolution.height);

  backgroundTexture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_IYUV, SDL_TEXTUREACCESS_STREAMING, resolution.width, resolution.height);
  target = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_RGBA8888, SDL_TEXTUREACCESS_TARGET, resolution.width, resolution.height);

  auto image = IMG_Load("../../assets/sprites.png");
  spriteSheet = SDL_CreateTextureFromSurface(renderer, image);
  SDL_FreeSurface(image);

  background = std::make_unique<BackgroundContext>(assets, resolution);
  backgroundTimerId = SDL_AddTimer(background->getInterval(), backgroundCallback, background.get());

  debugDraw = std::make_unique<DebugDrawDisplayAdapter>(renderer, Display::config);
  world->setDebugDraw(debugDraw.get());

  physics = std::make_unique<PhysicsContext>(&input, world);
}

Display::~Display() {
  SDL_RemoveTimer(backgroundTimerId);
  SDL_DestroyTexture(backgroundTexture);
  SDL_DestroyTexture(spriteSheet);
  SDL_DestroyTexture(target);
  SDL_DestroyRenderer(renderer);
  SDL_DestroyWindow(window);
  SDL_Quit();
}

bool Display::next() {
  while (SDL_PollEvent(&event) == 1) {
    switch (event.type) {
      case SDL_KEYUP:
      case SDL_KEYDOWN:
        if (assets->supportsCharacterSprite(event.key.keysym.sym)) {
          auto begin = input.keys.begin(), end = input.keys.end();
          auto it = std::find(begin, end, event.key.keysym.sym);

          if (event.type == SDL_KEYDOWN && it == end) {
            input.keys.push_back(event.key.keysym.sym);
          } else if (event.type == SDL_KEYUP && it != end) {
            input.keys.erase(it);
          }
        } else {
          switch (event.key.keysym.sym) {
            case SDLK_RIGHT:
              input.right = event.type == SDL_KEYDOWN;
              break;
            case SDLK_LEFT:
              input.left = event.type == SDL_KEYDOWN;
              break;
            case SDLK_UP:
              input.up = event.type == SDL_KEYDOWN;
              break;
            case SDLK_DOWN:
              input.down = event.type == SDL_KEYDOWN;
              break;
          }
        }
        break;

      case SDL_QUIT:
        input.quit = true;
        break;
    }
  }

  SDL_SetRenderTarget(renderer, target);

  SDL_SetRenderDrawColor(renderer, 0, 0, 0, SDL_ALPHA_OPAQUE);
  SDL_RenderClear(renderer);

  drawBackground();

  auto physicsLock = physics->getLock();
  if (SDL_SemWait(physicsLock) >= 0) {
    // we must sync updating and rendering physics state as a body could be destroyed as we're rendering it
    // TODO we don't have to lock both of these completely, just the but where bodies or model data is deleted
    drawWorld();
    SDL_SemPost(physicsLock);
  }

  SDL_SetRenderTarget(renderer, nullptr);
  SDL_SetRenderDrawColor(renderer, 0, 0, 0, SDL_ALPHA_OPAQUE);
  SDL_RenderClear(renderer);
  SDL_RenderCopyEx(renderer, target, nullptr, nullptr, 0, nullptr, SDL_FLIP_NONE);
  SDL_RenderPresent(renderer);

  return !input.quit;
}

void Display::drawBackground() {
  auto frame = background->getFrame();
  SDL_UpdateYUVTexture(backgroundTexture,
                       nullptr,
                       frame->data[0],
                       frame->linesize[0],
                       frame->data[1],
                       frame->linesize[1],
                       frame->data[2],
                       frame->linesize[2]);
  SDL_RenderCopy(renderer, backgroundTexture, nullptr, nullptr);
}

void Display::drawWorld() {
  if (config.debugPhysics) {
    world->debugDraw();
  }

  for (const auto& sprite : world->getSprites()) {
    auto model = sprite.model;
    if (model->getDefinition()->getType() == ModelType_Ground) {
      // TODO render ground
      continue;
    }

    auto angle = -(180 * sprite.angle / M_PIf32);
    auto size = model->getSize();

    auto ratio = std::max(1.f, size.width / size.height);
    auto rect = SDL_FRect {
      .x = X(sprite.position.x),
      .y = Y(sprite.position.y + ratio * size.height),
      .w = D(size.width),
      .h = D(size.height),
    };
    const auto asset = model->getAsset();
    auto src = SDL_Rect {
      .x = (int) asset->position.x,
      .y = (int) asset->position.y,
      .w = (int) asset->size.width,
      .h = (int) asset->size.height
    };

    // TODO why does this even work?!
    auto center = SDL_FPoint { .x = 0, .y = ratio * rect.h };
    SDL_RenderCopyExF(renderer, spriteSheet, &src, &rect, angle, &center, SDL_FLIP_NONE);
  }

  for (const auto& destroyedSprite : world->getDestroyedSprites()) {
    auto colour = destroyedSprite.model->getAsset()->colour;
    auto alpha = 255 * std::min(1.f, destroyedSprite.percentRemaining * 10 / 3);

    for (const auto& particle : destroyedSprite.particles) {
      auto vertices = particle.getVertices();
      Sint16 vx[3], vy[3];
      for (auto i = 0; i < 3; i++) {
        vx[i] = X(vertices.at(i).x);
        vy[i] = Y(vertices.at(i).y);
      }
      filledPolygonRGBA(renderer, vx, vy, 3, colour.r, colour.g, colour.b, alpha);
    }
  }
}
