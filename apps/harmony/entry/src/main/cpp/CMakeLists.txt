# NAPI 封装：把 Rust 的 C ABI 包成 ArkTS 能 import 的模块 `libkasm.so`。
cmake_minimum_required(VERSION 3.5.0)
project(kasm)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(NATIVERENDER_ROOT_PATH ${CMAKE_CURRENT_SOURCE_DIR})

include_directories(${NATIVERENDER_ROOT_PATH})

add_library(kasm SHARED napi_init.cpp)
target_link_libraries(kasm PUBLIC libace_napi.z.so
    ${NATIVERENDER_ROOT_PATH}/../../../libs/${OHOS_ARCH}/libkasm_harmony_core.a)
