#include <jni.h>

#include <ReactCommon/CxxTurboModuleUtils.h>

#include <CxxBasicModule.hpp>

jint JNI_OnLoad(JavaVM *vm, void *reserved) {
  facebook::react::registerCxxModuleToGlobalModuleMap(
      craby::basicmodule::CxxBasicModule::kModuleName,
      [](std::shared_ptr<facebook::react::CallInvoker> jsInvoker) {
        return std::make_shared<craby::basicmodule::CxxBasicModule>(jsInvoker);
      });
  return JNI_VERSION_1_6;
}
