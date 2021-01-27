#pragma once

#include <vector>

struct InputState {
    bool quit;
    bool up;
    bool down;
    bool left;
    bool right;
    std::vector<char> keys;
};