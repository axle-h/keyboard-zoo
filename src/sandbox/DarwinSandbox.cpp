#include "DarwinSandbox.h"

#include <ApplicationServices/ApplicationServices.h>

static CFRunLoopRef runLoop;

CGEventRef sandboxCallback(CGEventTapProxy proxy, CGEventType type, CGEventRef event, void *userInfo) {
  auto logger = (Logger*) userInfo;
  auto modifiers = CGEventGetFlags(event);

  auto cmd = (modifiers & kCGEventFlagMaskCommand) > 0;
  auto opt =  (modifiers & kCGEventFlagMaskAlternate)  > 0;

  if (!cmd && !opt) {
    // no modifier keys pressed, let mac os -> sdl handle the event.
    return event;
  }

  auto keycode = (CGKeyCode) CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);
  if (cmd && opt && keycode == 53) {
    // special case for ⌘ + ⌥ + ESC, we want to let this through so we have way of quiting
    if (type == kCGEventKeyDown) {
      logger->info("Received ⌘ + ⌥ + ESC, exiting");
    }
    return event;
  } else {
    // otherwise we should block any key when ⌘ or ⌥ are also pressed,
    // this will prevent context switching actions e.g. spotlight from breaking the sandbox.
    logger->info("[Sanboxed Event] type: {} modifiers: {} keycode: {}", type, modifiers, keycode);
    return nullptr;
  }
}

int sandboxThread(void *context) {
  auto eventMask = CGEventMaskBit(kCGEventFlagsChanged) | CGEventMaskBit(kCGEventKeyDown) | CGEventMaskBit(kCGEventKeyUp);
  auto eventTap = CGEventTapCreate(kCGSessionEventTap, kCGHeadInsertEventTap, kCGEventTapOptionDefault, eventMask, sandboxCallback, context);
  auto runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, eventTap, 0);
  CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource, kCFRunLoopCommonModes);
  CGEventTapEnable(eventTap, true);

  runLoop = CFRunLoopGetCurrent();

  CFRunLoopRun();
  return 0;
}

DarwinSandbox::DarwinSandbox(const std::shared_ptr<Config>& config, const std::shared_ptr<Logger>& logger)
  : config(config->getSandbox()), logger(logger) {
  if (this->config.enabled) {
    logger->info("starting mac os sandbox");

    if (runLoop) {
      throw std::runtime_error("Sandbox run loop already started, there can only ever be a single instance of the sandbox");
    }

    thread = SDL_CreateThread(sandboxThread, "DarwinSandbox", logger.get());
  } else {
    logger->warn("not running sandbox, app not toddler safe!");
  }
}

DarwinSandbox::~DarwinSandbox() {
  logger->info("stopping mac os sandbox");
  if (runLoop) {
    CFRunLoopStop(runLoop);
    if (thread) {
      SDL_WaitThread(thread, nullptr);
    }
  }
}
