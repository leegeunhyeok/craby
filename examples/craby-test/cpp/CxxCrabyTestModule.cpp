#include "CxxCrabyTestModule.hpp"
#include "ffi.rs.h"

using namespace facebook;

namespace craby::crabytest {

CxxCrabyTestModule::CxxCrabyTestModule(std::shared_ptr<react::CallInvoker> jsInvoker)
    : TurboModule(CxxCrabyTestModule::kModuleName, jsInvoker) {

  methodMap_["numericMethod"] =
      MethodMetadata{1, &CxxCrabyTestModule::JSI__numericMethod};
  methodMap_["booleanMethod"] =
      MethodMetadata{1, &CxxCrabyTestModule::JSI__booleanMethod};
  methodMap_["stringMethod"] =
      MethodMetadata{1, &CxxCrabyTestModule::JSI__stringMethod};

  callInvoker_ = std::move(jsInvoker);
}

jsi::Value CxxCrabyTestModule::JSI__numericMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxCrabyTestModule &>(turboModule);
  if (1 == count && args[0].isNumber()) {
    auto arg0 = args[0].asNumber();
    auto ret = craby::codegen::crabytest::numericMethod(arg0);
    return jsi::Value(ret);
  }

  throw jsi::JSError(rt, "Expected 1 argument (number)");
}

jsi::Value CxxCrabyTestModule::JSI__booleanMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxCrabyTestModule &>(turboModule);
  if (1 == count && args[0].isBool()) {
    auto arg0 = args[0].asBool();
    auto ret = craby::codegen::crabytest::booleanMethod(arg0);
    return jsi::Value(ret);
  }

  throw jsi::JSError(rt, "Expected 1 argument (boolean)");
}

jsi::Value CxxCrabyTestModule::JSI__stringMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxCrabyTestModule &>(turboModule);
  if (1 == count && args[0].isString()) {
    auto arg0 = args[0].asString(rt).utf8(rt).c_str();
    auto ret = jsi::String::createFromUtf8(rt, std::string(craby::codegen::crabytest::stringMethod(arg0)));
    return jsi::Value(rt, ret);
  }

  throw jsi::JSError(rt, "Expected 1 argument (string)");
}

} // namespace craby::crabytest
