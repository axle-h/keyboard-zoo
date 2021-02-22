#pragma once

#include "../assets/Assets.h"
#include "FrameService.h"
#include "VideoScaler.h"

class VideoContext {
  std::shared_ptr<Assets> assets;
  Resolution resolution;
  std::unique_ptr<FrameService> frameService;
  std::unique_ptr<VideoScaler> frameScaler;

  void nextVideo();

public:
  VideoContext(std::shared_ptr<Assets> assets, Resolution resolution);
  [[nodiscard]] uint getInterval() const;
  uint update();
  [[nodiscard]] AVFrame *getFrame() const;
};

uint updateVideoContextCallback(uint interval, void *params);
