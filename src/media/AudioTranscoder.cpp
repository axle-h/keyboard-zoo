#include "AudioTranscoder.h"

#include <utility>
#include <filesystem>

#include "wrappers/PacketRef.h"
#include "wrappers/FrameRef.h"
#include "wrappers/SamplesRef.h"

extern "C" {
  #include "libavutil/avstring.h"
  #include "libavutil/opt.h"
}

namespace fs = std::filesystem;

static const auto OUTPUT_BIT_RATE = 96000;
static const auto OUTPUT_CHANNELS = 2;

static char *get_error_text(const int error) {
  static char buffer[255];
  av_strerror(error, buffer, sizeof(buffer));
  return buffer;
}

AudioTranscoder::AudioTranscoder(std::string filename, std::shared_ptr<Logger> logger, FrameService *source)
    : filename(std::move(filename)), logger(std::move(logger)), source(source), sourceFormat(source->getAudioFormat()) {}

AudioTranscoder::~AudioTranscoder() {
  if (resample_context) {
    swr_close(resample_context);
    swr_free(&resample_context);
  }

  if (codecContext) {
    avcodec_free_context(&codecContext);
  }

  if (formatContext) {
    avio_close(formatContext->pb);
    avformat_free_context(formatContext);
  }
}

bool AudioTranscoder::initOutput(std::string filename) {
  AVIOContext *ioContext = nullptr;
  int error;

  // Open the output file to write to it.
  if ((error = avio_open(&ioContext, filename.c_str(), AVIO_FLAG_WRITE)) < 0) {
    logger->error("Could not open output file '{}' (error '{}')", filename, get_error_text(error));
    return false;
  }

  // Create a new format context for the output container format.
  if (!(formatContext = avformat_alloc_context())) {
    logger->error("Could not allocate output format context");
    return false;
  }

  // Associate the output file (pointer) with the container format context.
  formatContext->pb = ioContext;

  // Guess the desired container format based on the file extension.
  if (!(formatContext->oformat = av_guess_format(nullptr, filename.c_str(), nullptr))) {
    logger->error("Could not find output file format");
    return false;
  }

  auto codecId = formatContext->oformat->audio_codec;

  // Find the encoder to be used by its name.
  auto codec = avcodec_find_encoder(codecId);
  if (!codec) {
    logger->error("Could not find a {} audio codec", codecId);
    return false;
  }

  // Create a new audio stream in the output file container.
  auto stream = avformat_new_stream(formatContext, nullptr);
  if (!stream) {
    logger->error("Could not create new stream");
    return false;
  }

  codecContext = avcodec_alloc_context3(codec);
  if (!codecContext) {
    logger->error("Could not allocate an encoding context");
    return false;
  }

  /* Set the basic encoder parameters.
   * The input file's sample rate is used to avoid a sample rate conversion. */
  codecContext->channels       = OUTPUT_CHANNELS;
  codecContext->channel_layout = av_get_default_channel_layout(OUTPUT_CHANNELS);
  codecContext->sample_rate    = sourceFormat.sampleRate;
  codecContext->sample_fmt     = codec->sample_fmts[0];
  codecContext->bit_rate       = OUTPUT_BIT_RATE;

  // Allow the use of the experimental AAC encoder.
  codecContext->strict_std_compliance = FF_COMPLIANCE_EXPERIMENTAL;

  // Set the sample rate for the container.
  stream->time_base.den = sourceFormat.sampleRate;
  stream->time_base.num = 1;

  // Some container formats (like MP4) require global headers to be present.
  // Mark the encoder so that it behaves accordingly.
  if (formatContext->oformat->flags & AVFMT_GLOBALHEADER) {
    codecContext->flags |= AV_CODEC_FLAG_GLOBAL_HEADER;
  }

  // Open the encoder for the audio stream to use it later.
  if ((error = avcodec_open2(codecContext, codec, nullptr)) < 0) {
    logger->error("Could not open output codec (error '{}')", get_error_text(error));
    return false;
  }

  if (codecContext->frame_size == 0) {
    codecContext->frame_size = 4;
  }

  if (avcodec_parameters_from_context(stream->codecpar, codecContext) < 0) {
    logger->error("Could not initialize stream parameters");
    return false;
  }

  return true;
}

bool AudioTranscoder::initResampler() {
  // Only initialize the resampler if it is necessary, i.e. if and only if the sample formats differ.
  if (sourceFormat.sampleFormat == codecContext->sample_fmt && sourceFormat.channels == codecContext->channels) {
    return true;
  }
  // Create a resampler context for the conversion.
  if (!(resample_context = swr_alloc())) {
    logger->error("Could not allocate resample context");
    return false;
  }

  // Set the conversion parameters.
  // Default channel layouts based on the number of channels are assumed for simplicity (they are sometimes not detected properly by the demuxer and/or decoder).
  av_opt_set_int(resample_context, "in_channel_layout", av_get_default_channel_layout(sourceFormat.channels), 0);
  av_opt_set_int(resample_context, "out_channel_layout", av_get_default_channel_layout(codecContext->channels), 0);
  av_opt_set_int(resample_context, "in_sample_rate", sourceFormat.sampleRate, 0);
  av_opt_set_int(resample_context, "out_sample_rate", codecContext->sample_rate, 0);
  av_opt_set_int(resample_context, "in_sample_fmt", sourceFormat.sampleFormat, 0);
  av_opt_set_int(resample_context, "out_sample_fmt", codecContext->sample_fmt, 0);

  // Open the resampler with the specified parameters.
  if (swr_init(resample_context) < 0) {
    logger->error("Could not open resample context");
    return false;
  }

  return true;
}

bool AudioTranscoder::writeHeader() {
  int error;
  if ((error = avformat_write_header(formatContext, nullptr)) < 0) {
    logger->error("Could not write output file header (error '{}')", get_error_text(error));
    return false;
  }
  return true;
}

void AudioTranscoder::init() {
  if (!initOutput(filename) || !initResampler() || !writeHeader()) {
    throw std::runtime_error("Could not initialise audio writer");
  }

  fifo = std::make_unique<AudioFifo>(codecContext);
}

bool AudioTranscoder::write() {
  auto frame = source->getFrame();

  while (!source->isEof()) {
    // Make sure that there is one frame worth of samples in the FIFO buffer so that the encoder can do its work.
    // Since the decoder's and the encoder's frame size may differ, we need to FIFO buffer to store as many frames worth of input samples that they make up at least one frame worth of output samples.
    while (!fifo->isFull() && source->tryGetNext()) {
      // Temporary storage for the converted input samples.
      // Allocate as many pointers as there are audio channels.
      // Each pointer will later point to the audio samples of the corresponding channels (although it may be NULL for interleaved formats).
      auto samples = std::make_unique<SamplesRef>(codecContext, frame->nb_samples);

      // Convert the samples using the resampler.
      int error;
      if ((error = swr_convert(resample_context, samples->get(), frame->nb_samples, (const uint8_t **) frame->extended_data, frame->nb_samples)) < 0) {
        logger->error("Could not convert input samples (error '{}')", get_error_text(error));
        return false;
      }

      fifo->write(samples.get());
    }


    // If we have enough samples for the encoder, we encode them.
    // At the end of the file, we pass the remaining samples to the encoder.
    while (fifo->isFull() || (source->isEof() && !fifo->isEmpty())) {
      // Take one frame worth of audio samples from the FIFO buffer, encode it and write it to the output file.
      // Use the maximum number of possible samples per frame.
      // If there is less than the maximum possible frame size in the FIFO buffer use this number. Otherwise, use the maximum possible frame size.
      const int size = std::min(fifo->getSamplesAvailable(), codecContext->frame_size);

      // Temporary storage of the output samples of the frame written to the file.
      auto tempFrame = std::make_unique<FrameRef>(codecContext, size);
      fifo->read(tempFrame->get());

      // Encode one frame worth of audio samples.
      if (!encodeFrame(tempFrame->get())) {
        break;
      }
    }
  }

  // Flush the encoder as it may have delayed frames.
  while (encodeFrame(nullptr)) {}

  // Write the trailer of the output file container.
  return writeTrailer();
}

bool AudioTranscoder::encodeFrame(AVFrame *frame) {
  // Packet used for temporary storage.
  auto packet = std::make_unique<PacketRef>();

  // Set a timestamp based on the sample rate for the container.
  if (frame) {
    frame->pts = pts;
    pts += frame->nb_samples;
  }

  // Send the audio frame stored in the temporary packet to the encoder.
  // The output audio stream encoder is used to do this.
  auto error = avcodec_send_frame(codecContext, frame);
  if (error == AVERROR_EOF) {
    // The encoder signals that it has nothing more to encode.
    return false;
  } else if (error < 0) {
    logger->error("Could not send packet for encoding (error '{}')", get_error_text(error));
    return false;
  }

  // Receive one encoded frame from the encoder.
  error = avcodec_receive_packet(codecContext, packet->get());
  if (error == AVERROR(EAGAIN) || error == AVERROR_EOF) {
    // If the encoder asks for more data to be able to provide an encoded frame, return indicating that no data is present.
    // Or the last frame has been encoded, stop encoding.
    return false;
  } else if (error < 0) {
    logger->error("Could not encode frame (error '{}')", get_error_text(error));
    return false;
  } else {
    // Default case: Return encoded data.
    if ((error = av_write_frame(formatContext, packet->get())) < 0) {
      logger->error("Could not write frame (error '{}')", get_error_text(error));
      return false;
    }
    return true;
  }
}

bool AudioTranscoder::writeTrailer() {
  auto error = av_write_trailer(formatContext);
  if (error < 0) {
    logger->error("Could not write output file trailer (error '{}')", get_error_text(error));
    return false;
  }
  return true;
}
