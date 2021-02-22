#pragma once

#include "../models/Geom.h"
#include "Formats.h"

extern "C" {
  #include <libavformat/avformat.h>
  #include <libswscale/swscale.h>
}

class VideoScaler {
  SwsContext *swsContext = nullptr;
  AVFrame *scaledFrame = nullptr;
  VideoFormat sourceFormat;
  int scaledWidth;
  int scaledHeight;

public:
  VideoScaler(const VideoFormat &sourceFormat, int scaledWidth, int scaledHeight);
  ~VideoScaler();

  void setSourceFormat(const VideoFormat &sourceFormat);

  [[nodiscard]] AVFrame *getScaledFrame() const;

  void scale(AVFrame *frame) const;
};
