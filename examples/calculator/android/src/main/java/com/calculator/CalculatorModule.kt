package com.calculator

import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.annotations.ReactModule

@ReactModule(name = CalculatorModule.NAME)
class CalculatorModule(reactContext: ReactApplicationContext) :
  NativeCalculatorSpec(reactContext) {

  init {
    System.loadLibrary("crabycalculator")
  }

  private external fun nativeMultiply(a: Double, b: Double): Double

  override fun getName(): String {
    return NAME
  }

  // Example method
  // See https://reactnative.dev/docs/native-modules-android
  override fun multiply(a: Double, b: Double): Double {
    return nativeMultiply(a, b);
  }

  companion object {
    const val NAME = "Calculator"
  }
}
