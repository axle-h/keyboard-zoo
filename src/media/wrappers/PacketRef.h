#pragma once

extern "C" {
  #include "libavcodec/avcodec.h"
}

class PacketRef {
  AVPacket packet{};

public:
  PacketRef();

  ~PacketRef();

  inline AVPacket *get() {
    return &packet;
  }
};