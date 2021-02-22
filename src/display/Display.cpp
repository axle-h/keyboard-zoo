#include "Display.h"
#include "DebugDrawDisplayAdapter.h"
#include <SDL2_gfxPrimitives.h>
#include <SDL_image.h>
#include <stdexcept>
#include <utility>

Display::Display(std::shared_ptr<Logger> logger, const std::shared_ptr<Config> &config,
                 const std::shared_ptr<Assets>& assets, const std::shared_ptr<World>& world, const std::shared_ptr<AudioService> &audio)
    : assets(assets), logger(std::move(logger)), config(config->getRender()), world(world), audio(audio) {
  input = std::make_shared<InputState>();

  if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_TIMER | SDL_INIT_AUDIO) < 0) {
    throw std::runtime_error("Cannot initialise SDL");
  }

  auto resolution = Display::config.internalResolution;

  window = SDL_CreateWindow(config->getTitle().c_str(),
                            SDL_WINDOWPOS_CENTERED,
                            SDL_WINDOWPOS_CENTERED,
                            resolution.width / 2, resolution.height / 2,
                            SDL_WINDOW_SHOWN | SDL_WINDOW_RESIZABLE);

  if (!window) {
    throw std::runtime_error("Cannot create SDL window");
  }

  if (Display::config.fullScreen) {
    auto targetDisplayMode = SDL_DisplayMode {
      .format = 0,  // don't care
      .w = resolution.width,
      .h = resolution.height,
      .refresh_rate = 0, // don't care
      .driverdata   = 0, // initialize to 0
    };

    SDL_DisplayMode displayMode;
    if (!SDL_GetClosestDisplayMode(0, &targetDisplayMode, &displayMode)) {
      throw std::runtime_error("No suitable display mode");
    }

    Display::logger->info("Display mode {}x{} @{}hz", displayMode.w, displayMode.h, displayMode.refresh_rate);

    SDL_SetWindowDisplayMode(window, &displayMode);
    SDL_SetWindowFullscreen(window, SDL_WINDOW_FULLSCREEN);
  }

  renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED | SDL_RENDERER_TARGETTEXTURE);

  if (!renderer) {
    throw std::runtime_error("Cannot create SDL renderer");
  }

  SDL_RenderSetLogicalSize(renderer, resolution.width, resolution.height);

  backgroundTexture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_IYUV, SDL_TEXTUREACCESS_STREAMING, resolution.width, resolution.height);
  target = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_RGBA8888, SDL_TEXTUREACCESS_TARGET, resolution.width, resolution.height);

  auto image = IMG_Load((config->getFilesystem().assets / "sprites.png").string().c_str());
  if (!image) {
    throw std::runtime_error("Cannot load sprite sheet");
  }
  spriteSheet = SDL_CreateTextureFromSurface(renderer, image);
  SDL_FreeSurface(image);

  background = std::make_unique<VideoContext>(assets, resolution);
  backgroundTimerId = SDL_AddTimer(background->getInterval(), updateVideoContextCallback, background.get());

  debugDraw = std::make_unique<DebugDrawDisplayAdapter>(renderer, Display::config);
  world->setDebugDraw(debugDraw.get());

  physics = std::make_unique<PhysicsContext>(input.get(), world.get());

  audio->init();
  audio->nextMusic();
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
  auto quit = false;

  while (SDL_PollEvent(&event) == 1) {
    switch (event.type) {
      case SDL_KEYUP:
      case SDL_KEYDOWN:
        switch (event.key.keysym.sym) {
          case SDLK_RIGHT:
            input->setRight(event.type == SDL_KEYDOWN);
            break;
          case SDLK_LEFT:
            input->setLeft(event.type == SDL_KEYDOWN);
            break;
          case SDLK_UP:
            input->setUp(event.type == SDL_KEYDOWN);
            break;
          case SDLK_DOWN:
            input->setDown(event.type == SDL_KEYDOWN);
            break;
          default:
            if (assets->supportsSprite(event.key.keysym.sym)) {
              input->setKey(event.key.keysym.sym, event.type == SDL_KEYDOWN);
            }
            break;
        }
        break;

      case SDL_QUIT:
        quit = true;
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
    // TODO we don't have to lock both of these completely, just the bit where bodies or model data is deleted
    drawWorld();
    SDL_SemPost(physicsLock);
  }

  SDL_SetRenderTarget(renderer, nullptr);
  SDL_SetRenderDrawColor(renderer, 0, 0, 0, SDL_ALPHA_OPAQUE);
  SDL_RenderClear(renderer);
  SDL_RenderCopyEx(renderer, target, nullptr, nullptr, 0, nullptr, SDL_FLIP_NONE);
  SDL_RenderPresent(renderer);

  return !quit;
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

  for (const auto &sprite : world->getSprites()) {
    auto model = sprite.model;
    if (model->getDefinition()->getType() == ModelType_Ground) {
      // TODO render ground
      continue;
    }

    auto angle = -(180 * sprite.angle / M_PI);
    auto size = model->getSize();

    auto ratio = std::max(1.f, size.width / size.height);
    auto rect = SDL_FRect{
      .x = X(sprite.position.x),
      .y = Y(sprite.position.y + ratio * size.height),
      .w = D(size.width),
      .h = D(size.height),
    };
    const auto asset = model->getAsset();
    auto assetPosition = asset->getPosition();
    auto assetSize = asset->getSize();
    auto src = SDL_Rect{
      .x = (int) assetPosition.x,
      .y = (int) assetPosition.y,
      .w = (int) assetSize.width,
      .h = (int) assetSize.height};

    // TODO why does this even work?!
    auto center = SDL_FPoint{.x = 0, .y = ratio * rect.h};
    SDL_RenderCopyExF(renderer, spriteSheet, &src, &rect, angle, &center, SDL_FLIP_NONE);

    if (!model->isCreated()) {
      model->setCreated();
      audio->playCreateSound(asset->getName());
    }
  }

  for (auto &explosion : world->getExplosions()) {
    if (!explosion.isDestroyed()) {
      explosion.setDestroyed();
      audio->playDestroySound();
    }

    auto colour = explosion.getAsset()->getColour();
    auto alpha = 255 * std::min(1.f, explosion.getPercent() * 10 / 3);

    for (const auto &particle : explosion.getParticles()) {
      auto vertices = particle.getVertices();

      Sint16 vx[vertices.size()], vy[vertices.size()];
      for (auto i = 0; i < vertices.size(); i++) {
        auto vertex = vertices.at(i);
        vx[i] = (Sint16) std::round(X(vertex.x));
        vy[i] = (Sint16) std::round(Y(vertex.y));
      }
      filledPolygonRGBA(renderer, vx, vy, vertices.size(), colour.r, colour.g, colour.b, alpha);
    }
  }
}
