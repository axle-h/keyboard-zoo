#include "VideoContext.h"
#include <iostream>
#include <memory>
#include <utility>

VideoContext::VideoContext(std::shared_ptr<Assets> assets, Resolution resolution)
  : assets(std::move(assets)), resolution(resolution) {
  nextVideo();
}

void VideoContext::nextVideo() {
  auto nextPath = assets->getRandomVideo();
  frameService = std::make_unique<FrameService>(nextPath, AVMEDIA_TYPE_VIDEO);
  auto format = frameService->getVideoFormat();

  if (frameScaler) {
    frameScaler->setSourceFormat(format);
  } else {
    frameScaler = std::make_unique<VideoScaler>(format, resolution.width, resolution.height);
  }
}

uint32 VideoContext::getInterval() const {
  auto format = frameService->getVideoFormat();
  auto interval = (uint32) std::ceil(1000.f / format.framerate);
  return interval > 0 ? interval : 1;
}

uint32 VideoContext::update() {
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

AVFrame *VideoContext::getFrame() const {
  return frameScaler->getScaledFrame();
}

uint32 updateVideoContextCallback(uint32 interval, void *params) {
  auto context = (VideoContext *) params;
  return context->update();
}
