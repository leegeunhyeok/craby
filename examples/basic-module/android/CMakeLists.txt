cmake_minimum_required(VERSION 3.13)

project(react-native-basic-module)

set (CMAKE_VERBOSE_MAKEFILE ON)
set (CMAKE_CXX_STANDARD 20)

find_package(ReactAndroid REQUIRED CONFIG)
find_package(hermes-engine REQUIRED CONFIG)

# Import the pre-built Craby library
add_library(basic-module-lib STATIC IMPORTED)
set_target_properties(basic-module-lib PROPERTIES
  IMPORTED_LOCATION "${CMAKE_SOURCE_DIR}/../libs/${ANDROID_ABI}/libbasicmodule.a"
)
target_include_directories(basic-module-lib INTERFACE
  "${CMAKE_SOURCE_DIR}/../libs/${ANDROID_ABI}/include"
)

# Generated C++ source files by the Craby
add_library(basic-module SHARED
  src/main/cpp/OnLoad.cpp
  ../cpp/CxxBasicModule.cpp
)
target_include_directories(basic-module PRIVATE
  ../cpp
)

target_link_libraries(basic-module
  # android
  ReactAndroid::reactnative
  ReactAndroid::jsi
  hermes-engine::libhermes
  # basic-module-lib
  basic-module-lib
)
