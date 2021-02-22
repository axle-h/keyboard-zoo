#pragma once

extern "C" {
  #include "libavcodec/avcodec.h"
}

class FrameRef {
  AVFrame *frame;

public:
  FrameRef(AVCodecContext *context, int samples);

  ~FrameRef();

  inline AVFrame *get() {
    return frame;
  }
};