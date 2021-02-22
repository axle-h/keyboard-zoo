#pragma once

#include "Formats.h"
#include <string>

extern "C" {
  #include <libavformat/avformat.h>
}

class FrameService {
  AVFormatContext *fmt_ctx = nullptr;
  AVCodecContext *dec_ctx;
  AVPacket *pkt;
  AVFrame *frame;
  int streamId;
  AVMediaType type;
  bool eof = false;

  bool tryGetNextPacket();
  bool tryGetNextFrame();

public:
  FrameService(std::string path, enum AVMediaType type);
  ~FrameService();

  bool tryGetNext();

  [[nodiscard]] AVFrame *getFrame() const;
  [[nodiscard]] VideoFormat getVideoFormat() const;
  [[nodiscard]] AudioFormat getAudioFormat() const;
  [[nodiscard]] inline bool isEof() const {
    return eof;
  }
};