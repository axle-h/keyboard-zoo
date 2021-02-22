#pragma once

#include <string>

#include "../logger/Logger.h"
#include "Formats.h"
#include "FrameService.h"
#include "wrappers/AudioFifo.h"

extern "C" {
  #include "libavformat/avformat.h"
  #include "libswresample/swresample.h"
}

class AudioTranscoder {
  std::string filename;
  std::shared_ptr<Logger> logger;
  FrameService *source;
  AVFormatContext *formatContext = nullptr;
  AVCodecContext *codecContext = nullptr;
  SwrContext *resample_context = nullptr;
  std::unique_ptr<AudioFifo> fifo;
  AudioFormat sourceFormat;
  long pts = 0;

  bool initOutput(std::string filename);
  bool initResampler();
  bool writeHeader();
  bool encodeFrame(AVFrame *frame);
  bool writeTrailer();

public:
  AudioTranscoder(std::string filename, std::shared_ptr<Logger> logger, FrameService *source);
  ~AudioTranscoder();

  void init();
  bool write();
};
