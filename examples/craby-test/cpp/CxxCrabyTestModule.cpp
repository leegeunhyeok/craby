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

jsi::Value CxxCrabyTestModule::JSI__objectMethod(jsi::Runtime &rt,
                                         react::TurboModule &turboModule,
                                         const jsi::Value args[],
                                         size_t count) {
  auto &thisModule = static_cast<CxxCrabyTestModule &>(turboModule);
  if (1 == count && args[0].isObject()) {
    auto arg0 = args[0].asObject(rt);

    auto testObject__foo = arg0.getProperty(rt, "foo");
    auto testObject__bar = arg0.getProperty(rt, "bar");
    auto testObject__baz = arg0.getProperty(rt, "baz");

    if (!(
      testObject__foo.isString() &&
      testObject__bar.isNumber() &&
      testObject__baz.isBool()
    )) {
      throw jsi::JSError(rt, "Invalid argument (TestObject)");
    }

    craby::codegen::crabytest::TestObject testObject = {
      testObject__foo.asString(rt).utf8(rt).c_str(),
      testObject__bar.asNumber(),
      testObject__baz.asBool()
    };

    auto ret = craby::codegen::crabytest::objectMethod(testObject);
    jsi::Object obj = jsi::Object(rt);
    obj.setProperty(rt, "foo", jsi::String::createFromUtf8(rt, ret.foo));
    obj.setProperty(rt, "bar", jsi::Value(ret.bar));
    obj.setProperty(rt, "baz", jsi::Value(ret.baz));

    return jsi::Value(rt, obj);
  }

  throw jsi::JSError(rt, "Expected 1 argument (string)");
}

} // namespace craby::crabytest
