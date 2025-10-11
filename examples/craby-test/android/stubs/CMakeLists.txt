cmake_minimum_required(VERSION 3.13)
set(CMAKE_VERBOSE_MAKEFILE on)

file(GLOB react_codegen_SRCS CONFIGURE_DEPENDS *.cpp)

add_library(
  react_codegen_CrabyTest_stub
  OBJECT
  ${react_codegen_SRCS}
)

target_include_directories(react_codegen_CrabyTest_stub PUBLIC .)

target_link_libraries(
  react_codegen_CrabyTest_stub
  fbjni
  jsi
  # We need to link different libraries based on whether we are building rncore or not, that's necessary
  # because we want to break a circular dependency between react_codegen_rncore and reactnative
  reactnative
)

target_compile_options(
  react_codegen_CrabyTest_stub
  PRIVATE
  -DLOG_TAG=\"ReactNative\"
  -fexceptions
  -frtti
  -std=c++20
  -Wall
)
