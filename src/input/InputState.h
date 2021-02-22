#pragma once

#include <set>

class InputState {
  bool up = false;
  bool down = false;
  bool left = false;
  bool right = false;
  std::set<char> keys;

public:
  inline bool getUp() const {
    return up;
  }

  inline void setUp(bool up) {
    InputState::up = up;
  }

  inline bool getDown() const {
    return down;
  }

  inline void setDown(bool down) {
    InputState::down = down;
  }

  inline bool getLeft() const {
    return left;
  }

  inline void setLeft(bool left) {
    InputState::left = left;
  }

  inline bool getRight() const {
    return right;
  }

  inline void setRight(bool right) {
    InputState::right = right;
  }

  inline std::set<char> getKeys() const {
    return std::set<char>(keys);
  }

  void setKey(char key, bool set) {
    if (set) {
      keys.insert(key);
    } else {
      keys.erase(key);
    }
  }
};