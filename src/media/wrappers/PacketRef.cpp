#include "PacketRef.h"

PacketRef::PacketRef() {
  av_init_packet(&packet);

  // Set the packet data and size so that it is recognized as being empty.
  packet.data = nullptr;
  packet.size = 0;
}

PacketRef::~PacketRef() {
  av_packet_unref(&packet);
}
