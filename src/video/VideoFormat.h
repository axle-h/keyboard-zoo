#pragma once

extern "C" {
  #include <libavutil/pixfmt.h>
}

struct VideoFormat {
  int width;
  int height;
  AVPixelFormat pixelFormat;
  float framerate;
};