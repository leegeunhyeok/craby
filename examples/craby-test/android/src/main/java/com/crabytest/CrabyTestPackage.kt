package com.crabytest

import android.util.Log
import com.facebook.react.BaseReactPackage
import com.facebook.react.bridge.NativeModule
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.model.ReactModuleInfo
import com.facebook.react.module.model.ReactModuleInfoProvider
import com.facebook.soloader.SoLoader

import java.util.HashMap

class CrabyTestPackage : BaseReactPackage() {
  init {
    SoLoader.loadLibrary("cxx-craby-test")
  }

  override fun getModule(name: String, reactContext: ReactApplicationContext): NativeModule? {
    nativeSetDataPath(reactContext.filesDir.absolutePath)
    return null
  }

  override fun getReactModuleInfoProvider(): ReactModuleInfoProvider {
    return ReactModuleInfoProvider {
      val moduleInfos: MutableMap<String, ReactModuleInfo> = HashMap()
      moduleInfos
    }
  }

  private external fun nativeSetDataPath(dataPath: String)
}
