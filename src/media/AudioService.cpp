#include "AudioService.h"
#include "FrameService.h"
#include "AudioTranscoder.h"
#include <filesystem>
#include <utility>

AudioService::AudioService(std::shared_ptr<Assets> assets, std::shared_ptr<Logger> logger)
    : assets(std::move(assets)), logger(std::move(logger)) {}

AudioService::~AudioService() {
  for (const auto& audio : create) {
    freeAudio(audio.second);
  }
  for (const auto& audio : destroy) {
    freeAudio(audio);
  }
  for (const auto& audio : music) {
    freeAudio(audio);
  }
}

void AudioService::init() {
#ifdef WIN32
  // I don't know why this is needed, SDL on MINGW64 must be selecting some nop audio driver by default.
  SDL_AudioInit("directsound");
#endif

  initAudio();

  for (const auto& asset : assets->getAudioAssets()) {
    const auto& path = asset.getDecompressedPath();

    if (!std::filesystem::exists(path)) {
      logger->info("Transcoding {} -> {}", asset.getCompressedPath(), path);
      auto reader = std::make_unique<FrameService>(asset.getCompressedPath(), AVMEDIA_TYPE_AUDIO);
      auto writer = std::make_unique<AudioTranscoder>(path, logger, reader.get());
      writer->init();
      writer->write();
    }

    auto audio = createAudio(path.c_str(), 1, SDL_MIX_MAXVOLUME);
    switch (asset.getType()) {
      case AudioAssetType::Create:
        create.emplace(asset.getName(), audio);
        break;

      case AudioAssetType::Destroy:
        destroy.push_back(audio);
        break;

      case AudioAssetType::Music:
        music.push_back(audio);
        break;
    }
  }
}

void AudioService::nextMusic() {
  // TODO currently only plays random track
  playMusicFromMemory(music.at(std::rand() % music.size()), SDL_MIX_MAXVOLUME / 2);
}

void AudioService::playCreateSound(std::string name) {
  auto it = create.find(name);
  if (it == create.end()) {
    return;
  }
  playSoundFromMemory(it->second, SDL_MIX_MAXVOLUME);
}

void AudioService::playDestroySound() {
  auto audio = destroy.at(std::rand() % destroy.size());
  playSoundFromMemory(audio, SDL_MIX_MAXVOLUME / 2);
}
