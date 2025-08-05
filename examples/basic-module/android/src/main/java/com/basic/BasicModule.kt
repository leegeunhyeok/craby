package com.basic

import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.annotations.ReactModule

@ReactModule(name = BasicModule.NAME)
class BasicModule(reactContext: ReactApplicationContext) :
  NativeBasicSpec(reactContext) {

  init {
    System.loadLibrary("basicmodule")
  }

  private external fun nativeNumericMethod(arg: Double): Double
  private external fun nativeBooleanMethod(arg: Boolean): Boolean

  override fun getName(): String {
    return NAME
  }

  override fun numericMethod(arg: Double): Double {
    return nativeNumericMethod(arg);
  }

  override fun booleanMethod(arg: Boolean): Boolean {
    return nativeBooleanMethod(arg);
  }

  companion object {
    const val NAME = "Basic"
  }
}
