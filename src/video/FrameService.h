#pragma once

#include "VideoFormat.h"
#include <string>

extern "C" {
#include <libavformat/avformat.h>
}

class FrameService {
  AVFormatContext *fmt_ctx = nullptr;
  AVCodecContext *dec_ctx;
  AVPacket *pkt;
  AVFrame *frame;
  int video_stream_idx;
  bool seeking = false;

  bool tryGetNextPacket();
  bool tryGetNextFrame();

public:
  explicit FrameService(std::string path);
  ~FrameService();

  bool tryGetNext();

  [[nodiscard]] AVFrame *getFrame() const;
  VideoFormat getFormat();
};