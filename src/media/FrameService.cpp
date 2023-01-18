#include "FrameService.h"

#include <iostream>
#include <sstream>

extern "C" {
  #include <libavcodec/avcodec.h>
}

FrameService::FrameService(const std::string path, enum AVMediaType type) : type(type) {

  // Open input file, and allocate format context
  if (avformat_open_input(&fmt_ctx, path.data(), nullptr, nullptr) < 0) {
    throw std::runtime_error("Could not open source file");
  }

  // Retrieve stream information
  if (avformat_find_stream_info(fmt_ctx, nullptr) < 0) {
    throw std::runtime_error("Could not find stream information");
  }

  const AVCodec *codec = nullptr;
  streamId = av_find_best_stream(fmt_ctx, type, -1, -1, &codec, 0);
  if (streamId < 0) {
    throw std::runtime_error("Could not find a suitable stream");
  }

  // Allocate a codec context for the decoder
  dec_ctx = avcodec_alloc_context3(codec);
  if (!dec_ctx) {
    std::stringstream ss;
    ss << "Failed to allocate the " << av_get_media_type_string(type) << " codec context";
    throw std::runtime_error(ss.str());
  }

  // Copy codec parameters from input stream to output codec context
  if (avcodec_parameters_to_context(dec_ctx, fmt_ctx->streams[streamId]->codecpar) < 0) {
    std::stringstream ss;
    ss << "Failed to copy " << av_get_media_type_string(type) << " codec parameters to decoder context";
    throw std::runtime_error(ss.str());
  }

  // Init the decoder
  if (avcodec_open2(dec_ctx, codec, nullptr) < 0) {
    std::stringstream ss;
    ss << "Failed to open " << av_get_media_type_string(type) << " codec";
    throw std::runtime_error(ss.str());
  }

  // Allocate a packet and frame
  frame = av_frame_alloc();
  if (!frame) {
    throw std::runtime_error("Could not allocate frame");
  }

  pkt = new AVPacket();
  av_init_packet(pkt);
  pkt->data = nullptr;
  pkt->size = 0;

  if (pkt->pos == -1 && !tryGetNextPacket()) {
    throw std::runtime_error("Could not read first packet");
  }
}

FrameService::~FrameService() {
  av_frame_free(&frame);
  if (pkt->data != nullptr || pkt->buf != nullptr) {
    av_packet_free(&pkt);
  }
  avformat_close_input(&fmt_ctx);
  avcodec_close(dec_ctx);
}

bool FrameService::tryGetNextPacket() {
  while (av_read_frame(fmt_ctx, pkt) >= 0) {
    // check if the packet belongs to a stream we are interested in, otherwise skip it
    if (pkt->stream_index != streamId) {
      av_packet_unref(pkt);
      continue;
    }

    if (avcodec_send_packet(dec_ctx, pkt) < 0) {
      std::cerr << "Error decoding packet" << std::endl;
      av_packet_unref(pkt);
      continue;
    }

    return true;
  }
  return false;
}

bool FrameService::tryGetNextFrame() {
  while (true) {
    auto ret = avcodec_receive_frame(dec_ctx, frame);
    switch (ret) {
      case AVERROR_EOF:
        // End of the packet, no frame.
        return false;
      case AVERROR(EAGAIN):
        // Try again...
        return false;
      default:
        // Skip the rest of this packet on error
        return ret >= 0;
    }
  }
}

bool FrameService::tryGetNext() {
  if (eof) {
    return false;
  }

  while (!tryGetNextFrame()) {
    av_packet_unref(pkt);
    if (!tryGetNextPacket()) {
      eof = true;
      return false;
    }
  }

  return true;
}

AVFrame *FrameService::getFrame() const {
  return frame;
}

VideoFormat FrameService::getVideoFormat() const {
  if (type != AVMEDIA_TYPE_VIDEO) {
    throw std::runtime_error("Cannot get video format for non-video source");
  }

  auto framerate = fmt_ctx->streams[streamId]->avg_frame_rate;
  return VideoFormat {
    .width = dec_ctx->width,
    .height = dec_ctx->height,
    .pixelFormat = dec_ctx->pix_fmt,
    .framerate = (float) framerate.num / (float) framerate.den,
  };
}
