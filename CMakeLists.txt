cmake_minimum_required(VERSION 3.13)
project(baby-smash)

set(CMAKE_CXX_STANDARD 17)
set(EXECUTABLE_OUTPUT_PATH ${PROJECT_BINARY_DIR}/bin)
set(LIBRARY_OUTPUT_PATH ${PROJECT_BINARY_DIR}/lib)
set(ASSETS_OUTPUT_PATH ${PROJECT_BINARY_DIR}/assets)

include(FetchContent)
include(FindPkgConfig)

# Find required sdl2 libraries
pkg_check_modules(SDL2 REQUIRED IMPORTED_TARGET
        sdl2
        SDL2_image>=2.0.0
        SDL2_gfx)

# Find required libav libraries
pkg_check_modules(LIBAV REQUIRED IMPORTED_TARGET
        libavformat
        libavcodec
        libavutil
        libswscale)

# Fetch Box2D library from github
FetchContent_Declare(box2d
        GIT_REPOSITORY https://github.com/erincatto/box2d
        GIT_TAG v2.4.1)
FetchContent_GetProperties(box2d)
if(NOT box2d_POPULATED)
    FetchContent_Populate(box2d)
    add_subdirectory(${box2d_SOURCE_DIR} ${box2d_BINARY_DIR} EXCLUDE_FROM_ALL)
endif()

# Fetch json library from github
FetchContent_Declare(json
        GIT_REPOSITORY https://github.com/ArthurSonzogni/nlohmann_json_cmake_fetchcontent
        GIT_TAG v3.9.1)
FetchContent_GetProperties(json)
if(NOT json_POPULATED)
    FetchContent_Populate(json)
    add_subdirectory(${json_SOURCE_DIR} ${json_BINARY_DIR} EXCLUDE_FROM_ALL)
endif()

# Fetch platform_folders library from github
FetchContent_Declare(platform_folders
        GIT_REPOSITORY https://github.com/sago007/PlatformFolders
        GIT_TAG 4.0.0)
FetchContent_GetProperties(platform_folders)
if(NOT platform_folders_POPULATED)
    FetchContent_Populate(platform_folders)
    add_subdirectory(${platform_folders_SOURCE_DIR} ${platform_folders_BINARY_DIR} EXCLUDE_FROM_ALL)
endif()

# Fetch spdlog library from github
FetchContent_Declare(spdlog
        GIT_REPOSITORY https://github.com/gabime/spdlog
        GIT_TAG v1.x)
FetchContent_GetProperties(spdlog)
if(NOT spdlog_POPULATED)
    FetchContent_Populate(spdlog)
    add_subdirectory(${spdlog_SOURCE_DIR} ${spdlog_BINARY_DIR} EXCLUDE_FROM_ALL)
endif()

file(GLOB_RECURSE SOURCE_FILES "${PROJECT_SOURCE_DIR}/src/*.h" "${PROJECT_SOURCE_DIR}/src/*.cpp")

add_executable(baby-smash ${SOURCE_FILES})

# Copy assets
file(COPY assets DESTINATION ${PROJECT_BINARY_DIR})

target_link_libraries(baby-smash
        PkgConfig::LIBAV
        PkgConfig::SDL2
        box2d
        nlohmann_json::nlohmann_json
        platform_folders
        spdlog)

install(TARGETS baby-smash DESTINATION ${PROJECT_NAME}/bin)
install(DIRECTORY assets DESTINATION ${PROJECT_NAME})