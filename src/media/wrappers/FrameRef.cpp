#include "FrameRef.h"

#include <stdexcept>

FrameRef::FrameRef(AVCodecContext *context, int samples) {
  /* Create a new frame to store the audio samples. */
  if (!(frame = av_frame_alloc())) {
    throw std::runtime_error("Cannot allocate frame");
  }

  // Set the frame's parameters, especially its size and format.
  // av_frame_get_buffer needs this to allocate memory for the audio samples of the frame.
  // Default channel layouts based on the number of channels are assumed for simplicity.
  frame->nb_samples     = samples;
  frame->channel_layout = context->channel_layout;
  frame->format         = context->sample_fmt;
  frame->sample_rate    = context->sample_rate;

  // Allocate the samples of the created frame. This call will make sure that the audio frame can hold as many samples as specified.
  if (av_frame_get_buffer(frame, 0) < 0) {
    av_frame_free(&frame);
    throw std::runtime_error("Could not allocate output frame samples");
  }
}
FrameRef::~FrameRef() {
  av_frame_free(&frame);
}
