// TODO: Codegen

#include "CxxBasicModule.hpp"
#include "libbasicmodule.h"

using namespace facebook;

namespace craby::basicmodule {

CxxBasicModule::CxxBasicModule(std::shared_ptr<react::CallInvoker> jsInvoker)
    : TurboModule(CxxBasicModule::kModuleName, jsInvoker) {

  methodMap_["numericMethod"] =
      MethodMetadata{1, &CxxBasicModule::JSI__numericMethod};
  methodMap_["booleanMethod"] =
      MethodMetadata{1, &CxxBasicModule::JSI__booleanMethod};

  callInvoker_ = std::move(jsInvoker);
}

jsi::Value CxxBasicModule::JSI__numericMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxBasicModule &>(turboModule);
  if (1 == count && args[0].isNumber() && args[1].isNumber()) {
    auto a = args[0].asNumber();
    return jsi::Value(numericMethod(a));
  }

  throw jsi::JSError(rt, "Expected 1 argument (number)");
}

jsi::Value CxxBasicModule::JSI__booleanMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxBasicModule &>(turboModule);
  if (1 == count && args[0].isBool()) {
    auto a = args[0].asBool();
    return jsi::Value(booleanMethod(a));
  }

  throw jsi::JSError(rt, "Expected 1 argument (boolean)");
}

} // namespace craby::basicmodule
