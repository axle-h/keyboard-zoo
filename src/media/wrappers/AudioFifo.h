#pragma once

#include <cstdint>
extern "C" {
  #include "libavutil/audio_fifo.h"
}

#include "FrameRef.h"
#include "SamplesRef.h"

class AudioFifo {
  AVAudioFifo *fifo;
  int maxFrameSize;

public:
  explicit AudioFifo(AVCodecContext *context);

  ~AudioFifo();

  void read(AVFrame *frame);
  void write(SamplesRef *samples);

  [[nodiscard]] bool isFull() const;

  [[nodiscard]] bool isEmpty() const;

  [[nodiscard]] int getSamplesAvailable() const;
};