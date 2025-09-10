#include "CxxCrabyTestModule.hpp"
#include "libcrabytest.h"

using namespace facebook;

namespace craby::crabytest {

CxxCrabyTestModule::CxxCrabyTestModule(std::shared_ptr<react::CallInvoker> jsInvoker)
    : TurboModule(CxxCrabyTestModule::kModuleName, jsInvoker) {

  methodMap_["numericMethod"] =
      MethodMetadata{1, &CxxCrabyTestModule::JSI__numericMethod};
  methodMap_["booleanMethod"] =
      MethodMetadata{1, &CxxCrabyTestModule::JSI__booleanMethod};

  callInvoker_ = std::move(jsInvoker);
}

jsi::Value CxxCrabyTestModule::JSI__numericMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxCrabyTestModule &>(turboModule);
  if (1 == count && args[0].isNumber()) {
    auto a = args[0].asNumber();
    return jsi::Value(numericMethod(a));
  }

  throw jsi::JSError(rt, "Expected 1 argument (number)");
}

jsi::Value CxxCrabyTestModule::JSI__booleanMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxCrabyTestModule &>(turboModule);
  if (1 == count && args[0].isBool()) {
    auto a = args[0].asBool();
    return jsi::Value(booleanMethod(a));
  }

  throw jsi::JSError(rt, "Expected 1 argument (boolean)");
}

} // namespace craby::crabytest
