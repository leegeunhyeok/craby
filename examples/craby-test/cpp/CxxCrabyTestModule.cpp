#include "CxxCrabyTestModule.hpp"
#include "ffi.rs.h"

using namespace facebook;

namespace craby {
namespace crabytest {

CxxCrabyTestModule::CxxCrabyTestModule(std::shared_ptr<react::CallInvoker> jsInvoker)
    : TurboModule(CxxCrabyTestModule::kModuleName, jsInvoker) {

  methodMap_["numericMethod"] =
      MethodMetadata{1, &CxxCrabyTestModule::numericMethod};
  methodMap_["booleanMethod"] =
      MethodMetadata{1, &CxxCrabyTestModule::booleanMethod};
  methodMap_["stringMethod"] =
      MethodMetadata{1, &CxxCrabyTestModule::stringMethod};
  methodMap_["objectMethod"] =
      MethodMetadata{1, &CxxCrabyTestModule::objectMethod};

  callInvoker_ = std::move(jsInvoker);
}

jsi::Value CxxCrabyTestModule::numericMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxCrabyTestModule &>(turboModule);
  if (1 == count && args[0].isNumber()) {
    auto arg0 = args[0].asNumber();
    auto ret = craby::ffi::numericMethod(arg0);
    return jsi::Value(ret);
  }

  throw jsi::JSError(rt, "Expected 1 argument (number)");
}

jsi::Value CxxCrabyTestModule::booleanMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxCrabyTestModule &>(turboModule);
  if (1 == count && args[0].isBool()) {
    auto arg0 = args[0].asBool();
    auto ret = craby::ffi::booleanMethod(arg0);
    return jsi::Value(ret);
  }

  throw jsi::JSError(rt, "Expected 1 argument (boolean)");
}

jsi::Value CxxCrabyTestModule::stringMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxCrabyTestModule &>(turboModule);
  if (1 == count && args[0].isString()) {
    auto arg0 = args[0].asString(rt).utf8(rt).c_str();
    auto ret = jsi::String::createFromUtf8(rt, std::string(craby::ffi::stringMethod(arg0)));
    return jsi::Value(rt, ret);
  }

  throw jsi::JSError(rt, "Expected 1 argument (string)");
}

jsi::Value CxxCrabyTestModule::objectMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxCrabyTestModule &>(turboModule);
  if (1 == count && args[0].isObject()) {
    auto arg0 = args[0].asObject(rt);
    auto __TestObject$foo = arg0.getProperty(rt, "foo");
    auto __TestObject$bar = arg0.getProperty(rt, "bar");
    auto __TestObject$baz = arg0.getProperty(rt, "baz");

    // Validator
    if (!(
      __TestObject$foo.isString() &&
      __TestObject$bar.isNumber() &&
      __TestObject$baz.isBool()
    )) {
      throw jsi::JSError(rt, "Invalid argument (TestObject)");
    }

    craby::ffi::TestObject testObject = {
      __TestObject$foo.asString(rt).utf8(rt).c_str(),
      __TestObject$bar.asNumber(),
      __TestObject$baz.asBool()
    };

    auto ret = craby::ffi::objectMethod(testObject);
    jsi::Object obj = jsi::Object(rt);
    obj.setProperty(rt, "foo", jsi::String::createFromUtf8(rt, ret.foo.c_str()));
    obj.setProperty(rt, "bar", jsi::Value(ret.bar));
    obj.setProperty(rt, "baz", jsi::Value(ret.baz));

    return jsi::Value(rt, obj);
  }

  throw jsi::JSError(rt, "Expected 1 argument (string)");
}

} // namespace crabytest
} // namespace craby
