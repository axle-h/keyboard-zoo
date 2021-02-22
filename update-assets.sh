#!/bin/bash
yarn --cwd asset-factory
yarn --cwd asset-factory start
cp asset-factory/dist/sprites.* assets/
if [ -d "cmake-build-debug" ]; then
  cp asset-factory/dist/sprites.* cmake-build-debug/assets/
fi