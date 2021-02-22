#pragma once

extern "C" {
  #include <libavutil/pixfmt.h>
  #include <libavutil/samplefmt.h>
}

struct VideoFormat {
  int width;
  int height;
  AVPixelFormat pixelFormat;
  float framerate;
};

struct AudioFormat {
  AVSampleFormat sampleFormat;
  int channels;
  int sampleRate;
};