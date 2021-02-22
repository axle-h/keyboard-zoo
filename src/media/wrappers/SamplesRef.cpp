#include "SamplesRef.h"
#include <stdexcept>

SamplesRef::SamplesRef(AVCodecContext *context, int count) : count(count) {
  if (av_samples_alloc(&samples, nullptr, context->channels, count, context->sample_fmt, 0) < 0) {
    throw std::runtime_error("Could not allocate sample buffers");
  }
}

SamplesRef::~SamplesRef() {
  av_freep(&samples);
}
