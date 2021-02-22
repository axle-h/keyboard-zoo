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
  [[nodiscard]] uint32 getInterval() const;
  uint32 update();
  [[nodiscard]] AVFrame *getFrame() const;
};

uint32 updateVideoContextCallback(uint32 interval, void *params);
