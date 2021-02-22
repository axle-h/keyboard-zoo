#include "World.h"
#include "../models/Character.h"
#include "../models/Ground.h"

#include <algorithm>
#include <box2d/box2d.h>
#include <utility>

const int VELOCITY_ITERATIONS = 6;
const int POSITION_ITERATIONS = 2;
const int MAX_COLLISIONS = 5;
const auto CHARACTER_SCALE = 1 / 75.f;
const auto DIRECTION_FORCE = 200.f;
const auto MAX_PARTICLE_VELOCITY = 6;

Sprite getSprite(b2Body *body) {
  auto position = body->GetPosition();
  auto angle = body->GetAngle();

  // normalise to -pi -> pi
  while (angle >= M_PI) {
    angle -= 2 * M_PI;
  }
  while (angle <= -M_PI) {
    angle += 2 * M_PI;
  }

  auto model = (Model *) body->GetUserData().pointer;
  auto center = body->GetWorldCenter();
  return Sprite(model, angle, { position.x, position.y }, { center.x, center.y });
}

b2AABB getAABB(b2Body *body) {
  auto aabb = b2AABB{
    .lowerBound = b2Vec2(FLT_MAX, FLT_MAX),
    .upperBound = b2Vec2(-FLT_MAX, -FLT_MAX),
  };
  for (auto fixture = body->GetFixtureList(); fixture; fixture = fixture->GetNext()) {
    aabb.Combine(aabb, fixture->GetAABB(0));
  }
  return aabb;
}

World::World(std::shared_ptr<Logger> logger, const std::shared_ptr<Config> &config, std::shared_ptr<Assets> assets)
    : config(config->getWorld()), assets(std::move(assets)), logger(std::move(logger)) {
  auto gravity = b2Vec2(0.0f, World::config.gravity);
  world = std::make_unique<b2World>(gravity);

  auto render = config->getRender();
  auto resolution = render.internalResolution;
  auto width = (float) resolution.width / render.pixelsPerMeter;
  auto height = (float) resolution.height / render.pixelsPerMeter;
  worldSize = { width, height };

  buildGroundBody(width / 2, 0, width, 0);// bottom
  buildGroundBody(0, height / 2, 0, height);
  buildGroundBody(width, height / 2, 0, height);
  buildGroundBody(width / 2, height, width, 0);

  world->SetContactListener(this);
}

World::~World() {
  for (auto body = world->GetBodyList(); body; body = body->GetNext()) {
    delete (Model *) body->GetUserData().pointer;
    world->DestroyBody(body);
  }
}

void World::BeginContact(b2Contact *contact) {
  auto body1 = contact->GetFixtureA()->GetBody();
  auto body2 = contact->GetFixtureB()->GetBody();

  for (auto body : {body1, body2}) {
    if (body->GetType() != b2_dynamicBody) {
      continue;
    }
    auto model = (Model *) body->GetUserData().pointer;
    model->recordCollision();
  }
}

void World::buildGroundBody(float x, float y, float width, float height) {
  b2BodyDef groundBodyDef;
  groundBodyDef.position.Set(x, y);
  groundBodyDef.userData.pointer = reinterpret_cast<uintptr_t>(new Model(new Ground(), Dimensions{width, height}, nullptr));
  b2Body *groundBody = world->CreateBody(&groundBodyDef);

  b2PolygonShape groundBox;
  groundBox.SetAsBox(width > 0 ? width / 2 : 0.1f, height > 0 ? height / 2 : 0.1f);

  groundBody->CreateFixture(&groundBox, 0.0f);
}


bool World::tryAddModel(ModelDefinition *definition) {
  const SpriteAsset *asset = nullptr;
  switch (definition->getType()) {
    case ModelType_Character: {
      auto ch = (Character *) definition;
      asset = assets->getSprite(ch->getValue());
      break;
    }

    case ModelType_Ground:
      throw std::runtime_error("cannot dynamically add a ground body");
  }
  auto spriteSize = asset->getSize();
  auto spriteWidth = spriteSize.width * CHARACTER_SCALE;
  auto spriteHeight = spriteSize.height * CHARACTER_SCALE;
  auto spriteScale = std::max(spriteHeight, spriteWidth);

  auto windowSize = b2Vec2(spriteWidth, spriteHeight);
  auto windowPosition = b2Vec2(
    std::rand() % (int) (worldSize.width - windowSize.x),
    std::rand() % (int) (worldSize.height - windowSize.y));
  auto window = b2AABB{.lowerBound = windowPosition, .upperBound = windowPosition + windowSize};
  auto deltaX = b2Vec2(windowSize.x / 2, 0.f);
  auto deltaY = b2Vec2(0.f, windowSize.y / 2);
  auto windowOverflow = false;

  auto aabbs = std::vector<b2AABB>();
  for (auto body = world->GetBodyList(); body; body = body->GetNext()) {
    if (body->GetType() != b2_dynamicBody) {
      continue;
    }

    aabbs.push_back(getAABB(body));
  }

  while (!aabbs.empty()) {
    auto windowCollision = false;
    for (auto aabb : aabbs) {
      if (b2TestOverlap(aabb, window)) {
        // move the window on
        window.upperBound += deltaX;
        if (window.upperBound.x > worldSize.width) {
          // back to the start of the row.
          window.upperBound.x = windowSize.x;
          window.upperBound += deltaY;

          if (windowOverflow && window.upperBound.y >= windowPosition.y) {
            // we looped through all possible windows but still didnt find a free space.
            return false;
          }

          if (window.upperBound.y > worldSize.height) {
            // back to the origin
            windowOverflow = true;
            window.upperBound.Set(windowSize.x, windowSize.y);
          }
        }
        window.lowerBound = window.upperBound - windowSize;
        windowCollision = true;
        break;
      }
    }

    if (!windowCollision) {
      break;
    }
  }

  auto model = new Model(definition, Dimensions{windowSize.x, windowSize.y}, asset);

  b2BodyDef bodyDef;
  bodyDef.type = b2_dynamicBody;
  bodyDef.position = window.lowerBound;
  bodyDef.userData.pointer = reinterpret_cast<uintptr_t>(model);
  b2Body *newBody = world->CreateBody(&bodyDef);

  b2FixtureDef fixtureDef;
  fixtureDef.density = 1.0f;
  fixtureDef.friction = 0.30f;
  fixtureDef.restitution = .5f;

  for (auto polygon : asset->getPolygons()) {
    for (auto& vertex : polygon.vertices) {
      vertex *= spriteScale;
    }
    b2PolygonShape shape;
    shape.Set(polygon.vertices.data(), polygon.vertices.size());
    fixtureDef.shape = &shape;
    newBody->CreateFixture(&fixtureDef);
  }

  // TODO these forces should be proportional to the body size
  auto xf = std::rand() % 3000 - 1500;
  auto yf = std::rand() % 3000;
  auto torque = std::rand() % 1000 - 500;
  newBody->ApplyForceToCenter(b2Vec2((float) xf, (float) yf), true);
  newBody->ApplyTorque(torque, true);
  return true;
}

void World::update(float delta, const InputState *input) {
  auto timeStep = delta / 1000;

  auto force = b2Vec2(0, 0);
  if (input->getRight()) {
    force.x += DIRECTION_FORCE;
  }
  if (input->getLeft()) {
    force.x -= DIRECTION_FORCE;
  }
  if (input->getUp()) {
    force.y += DIRECTION_FORCE;
  }
  if (input->getDown()) {
    force.y -= DIRECTION_FORCE;
  }

  for (auto i = 0; i < explosions.size(); i++) {
    auto explosion = &explosions.at(i);
    if (explosion->nextFrame(timeStep)) {
      logger->info("destroyed '{}'", explosion->getAsset()->getName());
      explosions.erase(explosions.begin() + i--);
    }
  }

  for (auto body = world->GetBodyList(); body; body = body->GetNext()) {
    if (body->GetType() != b2_dynamicBody) {
      continue;
    }

    auto model = (Model *) body->GetUserData().pointer;
    if (model->getCollisions() >= MAX_COLLISIONS) {
      auto particles = std::vector<Particle>();
      auto transform = body->GetTransform();
      for (auto fixture = body->GetFixtureList(); fixture; fixture = fixture->GetNext()) {
        auto shape = (b2PolygonShape*) fixture->GetShape();

        auto velocity = b2Vec2(
          std::rand() % (2 * MAX_PARTICLE_VELOCITY) - MAX_PARTICLE_VELOCITY / 2,
          std::rand() % (2 * MAX_PARTICLE_VELOCITY) - MAX_PARTICLE_VELOCITY / 2);
        auto angularVelocity = (float) (std::rand() % 25) / 1000;

        auto vertices = std::vector<b2Vec2>(shape->m_count);
        for (auto i = 0; i < shape->m_count; i++) {
          vertices.at(i) = shape->m_vertices[i];
        }

        particles.emplace_back(vertices, transform, velocity, angularVelocity);
      }
      explosions.emplace_back(model, particles);
      world->DestroyBody(body);
      delete model;
      continue;
    }

    if (force.Length() > 0) {
      body->ApplyForceToCenter(force, true);
    }
  }

  auto keys = input->getKeys();
  for (auto key : keys) {
    if (lastKeys.find(key) != lastKeys.end()) {
      // key has 'just' been pressed, ignore it...
      continue;
    }

    auto shape = new Character(key);
    if (!tryAddModel(shape)) {
      delete shape;
    }
  }
  lastKeys = keys;

  world->Step(timeStep, VELOCITY_ITERATIONS, POSITION_ITERATIONS);
}

std::vector<Sprite> World::getSprites() const {
  auto sprites = std::vector<Sprite>();
  for (auto body = world->GetBodyList(); body; body = body->GetNext()) {
    auto sprite = getSprite(body);
    sprites.push_back(sprite);
  }
  return sprites;
}

std::vector<SpriteExplosion> &World::getExplosions() {
  return explosions;
}

void World::debugDraw() const {
  world->DebugDraw();
}
