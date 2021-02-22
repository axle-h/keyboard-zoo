#pragma once

extern "C" {
  #include "libavcodec/avcodec.h"
}

class SamplesRef {
  uint8_t *samples = nullptr;
  int count;

public:
  SamplesRef(AVCodecContext *context, int count);

  ~SamplesRef();

  inline uint8_t **get() {
    return &samples;
  }

  [[nodiscard]] inline int getSampleCount() const {
    return count;
  }
};