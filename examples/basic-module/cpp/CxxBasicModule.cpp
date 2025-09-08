// TODO: Codegen

#include "CxxBasicModule.hpp"

using namespace facebook;

namespace craby::basicmodule {

CxxBasicModule::CxxBasicModule(std::shared_ptr<react::CallInvoker> jsInvoker)
    : TurboModule(CxxBasicModule::kModuleName, jsInvoker) {

  methodMap_["numericMethod"] =
      MethodMetadata{2, &CxxBasicModule::numericMethod};
  methodMap_["booleanMethod"] =
      MethodMetadata{2, &CxxBasicModule::booleanMethod};

  callInvoker_ = std::move(jsInvoker);
}

jsi::Value CxxBasicModule::numericMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxBasicModule &>(turboModule);
  if (2 == count && args[0].isNumber() && args[1].isNumber()) {
    // return foo(rt, args[0].asNumber(rt), args[1].asNumber(rt));
  }

  throw jsi::JSError(rt, "Expected 2 arguments (number, number)");
}

jsi::Value CxxBasicModule::booleanMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxBasicModule &>(turboModule);
  if (2 == count && args[0].isBool() && args[1].isBool()) {
    // return foo(rt, args[0].asBool(rt), args[1].asBool(rt));
  }

  throw jsi::JSError(rt, "Expected 2 arguments (boolean, boolean)");
}

} // namespace craby::basicmodule
