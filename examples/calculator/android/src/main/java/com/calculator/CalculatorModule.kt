package com.calculator

import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.annotations.ReactModule

@ReactModule(name = CalculatorModule.NAME)
class CalculatorModule(reactContext: ReactApplicationContext) :
  NativeCalculatorSpec(reactContext) {

  init {
    System.loadLibrary("crabycalculator")
  }

  private external fun nativeAdd(a: Double, b: Double): Double
  private external fun nativeSubtract(a: Double, b: Double): Double
  private external fun nativeMultiply(a: Double, b: Double): Double
  private external fun nativeDivide(a: Double, b: Double): Double

  override fun getName(): String {
    return NAME
  }

  override fun add(a: Double, b: Double): Double {
    return nativeAdd(a, b);
  }

  override fun subtract(a: Double, b: Double): Double {
    return nativeSubtract(a, b);
  }

  override fun multiply(a: Double, b: Double): Double {
    return nativeMultiply(a, b);
  }

  override fun divide(a: Double, b: Double): Double {
    return nativeDivide(a, b);
  }

  companion object {
    const val NAME = "Calculator"
  }
}
