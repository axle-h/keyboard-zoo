#include "BackgroundContext.h"
#include <iostream>

BackgroundContext::BackgroundContext(Assets *assets, Resolution resolution)
  : assets(assets), frameService(nullptr), frameScaler(nullptr), resolution(resolution) {
  nextVideo();
}

BackgroundContext::~BackgroundContext() {
  delete frameScaler;
  delete frameService;
}

void BackgroundContext::nextVideo() {
  delete frameService;
  auto nextPath = assets->getRandomBackground();
  frameService = new FrameService(nextPath);
  auto format = frameService->getFormat();

  if (frameScaler) {
    frameScaler->setSourceFormat(format);
  } else {
    frameScaler = new FrameScaler(format, resolution.width, resolution.height);
  }
}

uint BackgroundContext::getInterval() const {
  auto format = frameService->getFormat();
  auto interval = (uint) std::ceil(1000.f / format.framerate);
  return interval > 0 ? interval : 1;
}

uint BackgroundContext::callback() {
  if (!frameService->tryGetNext()) {
    nextVideo();
    if (!frameService->tryGetNext()) {
      std::cerr << "cannot render any background video frames" << std::endl;
      return 1;
    }
  }
  auto interval = getInterval();

  frameScaler->scale(frameService->getFrame());
  return interval;
}

AVFrame *BackgroundContext::getFrame() const {
  return frameScaler->getScaledFrame();
}

uint backgroundCallback(uint interval, void *params) {
  auto context = (BackgroundContext *) params;
  return context->callback();
}
