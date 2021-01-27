#pragma once

#include "../assets/Assets.h"
#include "FrameScaler.h"
#include "FrameService.h"

class BackgroundContext {
  Assets *assets;
  Resolution resolution;
  FrameService *frameService;
  FrameScaler *frameScaler;

  void nextVideo();

public:
  explicit BackgroundContext(Assets *assets, Resolution resolution);
  ~BackgroundContext();
  [[nodiscard]] uint getInterval() const;
  uint callback();
  [[nodiscard]] AVFrame *getFrame() const;
};

uint backgroundCallback(uint interval, void *params);
