#include "AudioService.h"
#include "FrameService.h"
#include <filesystem>
#include <utility>

AudioService::AudioService(std::shared_ptr<Assets> assets, std::shared_ptr<Logger> logger)
    : assets(std::move(assets)), logger(std::move(logger)) {}

AudioService::~AudioService() {
  for (const auto& audio : create) {
    Mix_FreeChunk(audio.second);
  }
  for (const auto& audio : destroy) {
    Mix_FreeChunk(audio);
  }
  for (const auto& audio : music) {
    Mix_FreeMusic(audio);
  }
  Mix_CloseAudio();
  Mix_Quit();
}

void AudioService::init() {
#ifdef WIN32
  // I don't know why this is needed, SDL on MINGW64 must be selecting some nop audio driver by default.
  SDL_AudioInit("directsound");
#endif

  if (Mix_Init(MIX_INIT_OGG) != MIX_INIT_OGG) {
    throw std::runtime_error("Cannot initialize SDL_mixer with ogg support");
  }

  // Initialize SDL_mixer
  if (Mix_OpenAudio( 22050, MIX_DEFAULT_FORMAT, 2, 4096 ) == -1 ) {
    std::stringstream ss;
    ss << "Cannot initialize SDL_mixer: " << "(" << Mix_GetError();
    throw std::runtime_error(ss.str());
  }

  for (const auto& asset : assets->getAudioAssets()) {
    const auto& path = asset.getPath();

    switch (asset.getType()) {
      case AudioAssetType::Create:
        create.emplace(asset.getName(), Mix_LoadWAV(path.c_str()));
        break;

      case AudioAssetType::Destroy:
        destroy.push_back(Mix_LoadWAV(path.c_str()));
        break;

      case AudioAssetType::Music:
        music.push_back(Mix_LoadMUS(path.c_str()));
        break;
    }
  }
}

void AudioService::nextMusic() {
  // TODO currently only plays random track
  if(Mix_PlayingMusic() != 0) {
    Mix_HaltMusic();
  }

  auto audio = music.at(std::rand() % music.size());
  if(Mix_PlayingMusic() == 0) {
    Mix_PlayMusic(audio, -1);
  }
}

void AudioService::playCreateSound(std::string name) {
  auto it = create.find(name);
  if (it == create.end()) {
    return;
  }
  Mix_PlayChannel(-1, it->second, 0);
}

void AudioService::playDestroySound() {
  auto audio = destroy.at(std::rand() % destroy.size());
  Mix_PlayChannel(-1, audio, 0);
}
