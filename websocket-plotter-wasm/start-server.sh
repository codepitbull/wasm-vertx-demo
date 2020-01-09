#!/bin/bash
set -e

CONFIG=release
mkdir -p www/pkg

rustup target add wasm32-unknown-unknown


if [ "${CONFIG}" = "release" ]
then
    wasm-pack build
else
    wasm-pack build --release
fi

cd www
npm install
npm start
