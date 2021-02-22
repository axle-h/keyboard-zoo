#pragma once

#include "../assets/Assets.h"
extern "C" {
  #include "audio.h"
}

class AudioService {
  std::shared_ptr<Logger> logger;
  std::shared_ptr<Assets> assets;
  std::unordered_map<std::string, Audio *> create;
  std::vector<Audio *> destroy;
  std::vector<Audio *> music;

public:
  AudioService(std::shared_ptr<Assets> assets, std::shared_ptr<Logger> logger);
  ~AudioService();

  void init();
  void nextMusic();
  void playCreateSound(std::string name);
  void playDestroySound();
};
