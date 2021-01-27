#pragma once

#include "../models/Geom.h"
#include "VideoFormat.h"
extern "C" {
  #include <libavformat/avformat.h>
  #include <libswscale/swscale.h>
}

class FrameScaler {
  SwsContext *swsContext;
  AVFrame *scaledFrame;
  VideoFormat sourceFormat;
  int scaledWidth;
  int scaledHeight;
public:
  FrameScaler(const VideoFormat &sourceFormat, int scaledWidth, int scaledHeight);
  ~FrameScaler();

  void setSourceFormat(const VideoFormat &sourceFormat);

  [[nodiscard]] AVFrame *getScaledFrame() const;

  void scale(AVFrame *frame) const;
};
