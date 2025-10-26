package com.crabytest

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
    if (name in JNI_PREPARE_MODULE_NAME) {
      nativeSetDataPath(reactContext.filesDir.absolutePath)
    }
    return null
  }

  override fun getReactModuleInfoProvider(): ReactModuleInfoProvider {
    return ReactModuleInfoProvider {
      val moduleInfos: MutableMap<String, ReactModuleInfo> = HashMap()
      JNI_PREPARE_MODULE_NAME.forEach { name ->
        moduleInfos[name] = ReactModuleInfo(
          name,
          name,
          false,  // canOverrideExistingModule
          false,  // needsEagerInit
          false,  // isCxxModule
          true,  // isTurboModule
        )
      }
      moduleInfos
    }
  }

  private external fun nativeSetDataPath(dataPath: String)

  companion object {
    val JNI_PREPARE_MODULE_NAME = setOf(
      "__crabyCalculator_JNI_prepare__",
      "__crabyCrabyTest_JNI_prepare__"
    )
  }
}
