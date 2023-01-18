#pragma once

#include "../assets/Assets.h"
extern "C" {
  #include "SDL_mixer.h"
}

class AudioService {
  std::shared_ptr<Logger> logger;
  std::shared_ptr<Assets> assets;
  std::unordered_map<std::string, Mix_Chunk *> create;
  std::vector<Mix_Chunk *> destroy;
  std::vector<Mix_Music *> music;

public:
  AudioService(std::shared_ptr<Assets> assets, std::shared_ptr<Logger> logger);
  ~AudioService();

  void init();
  void nextMusic();
  void playCreateSound(std::string name);
  void playDestroySound();
};
