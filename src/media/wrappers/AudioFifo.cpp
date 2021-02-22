#include "AudioFifo.h"

#include <stdexcept>

AudioFifo::AudioFifo(AVCodecContext *context) : maxFrameSize(context->frame_size) {
  // Create the FIFO buffer based on the specified output sample format.
  if (!(fifo = av_audio_fifo_alloc(context->sample_fmt, context->channels, 1))) {
    throw std::runtime_error("Could not allocate FIFO");
  }
}

AudioFifo::~AudioFifo() {
  av_audio_fifo_free(fifo);
}

void AudioFifo::read(AVFrame *frame) {
  // Read as many samples from the FIFO buffer as required to fill the frame.
  // The samples are stored in the frame temporarily.
  if (av_audio_fifo_read(fifo, (void **) frame->data, frame->nb_samples) < frame->nb_samples) {
    throw std::runtime_error( "Could not read data from FIFO");
  }
}

void AudioFifo::write(SamplesRef *samples) {
  auto sampleCount = samples->getSampleCount();

  // Make the FIFO as large as it needs to be to hold both, the old and the new samples.
  if (av_audio_fifo_realloc(fifo, av_audio_fifo_size(fifo) + sampleCount) < 0) {
    throw std::runtime_error("Could not reallocate FIFO");
  }

  // Store the new samples in the FIFO buffer.
  if (av_audio_fifo_write(fifo, (void **) samples->get(), sampleCount) < sampleCount) {
    throw std::runtime_error("Could not write data to FIFO");
  }
}

bool AudioFifo::isFull() const {
  return av_audio_fifo_size(fifo) >= maxFrameSize;
}

bool AudioFifo::isEmpty() const {
  return av_audio_fifo_size(fifo) == 0;
}

int AudioFifo::getSamplesAvailable() const {
  return av_audio_fifo_size(fifo);
}
