package com.{{ flat_name }}

import android.app.ActivityThread
import com.facebook.react.BaseReactPackage
import com.facebook.react.bridge.NativeModule
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.model.ReactModuleInfo
import com.facebook.react.module.model.ReactModuleInfoProvider
import com.facebook.soloader.SoLoader

import java.util.HashMap

class {{ pascal_name }}Package : BaseReactPackage() {
  init {
    SoLoader.loadLibrary("cxx-{{ kebab_name }}")
  }

  override fun getModule(name: String, reactContext: ReactApplicationContext): NativeModule? {
    nativeSetDataPath(reactContext.filesDir.absolutePath)
    return null
  }

  override fun getReactModuleInfoProvider(): ReactModuleInfoProvider {
    return ReactModuleInfoProvider {
      val moduleInfos: MutableMap<String, ReactModuleInfo> = HashMap()
      moduleInfos[{{ pascal_name }}Package.JNI_PREPARE_MODULE_NAME] = ReactModuleInfo(
        {{ pascal_name }}Package.JNI_PREPARE_MODULE_NAME,
        {{ pascal_name }}Package.JNI_PREPARE_MODULE_NAME,
        false,  // canOverrideExistingModule
        false,  // needsEagerInit
        false,  // isCxxModule
        true,  // isTurboModule
      )
      moduleInfos
    }
  }

  private external fun nativeSetDataPath(dataPath: String)

  companion object {
    const val JNI_PREPARE_MODULE_NAME = "__crabyUnknown_JNI_prepare__"
  }
}
