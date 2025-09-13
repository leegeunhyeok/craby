#pragma once

#include <ReactCommon/TurboModule.h>
#include <jsi/jsi.h>

namespace craby {
namespace crabytest {

class JSI_EXPORT CxxCrabyTestModule : public facebook::react::TurboModule {
public:
  static constexpr const char *kModuleName = "CrabyTest";

  CxxCrabyTestModule(std::shared_ptr<facebook::react::CallInvoker> jsInvoker);

  static facebook::jsi::Value
  JSI__numericMethod(facebook::jsi::Runtime &rt,
                facebook::react::TurboModule &turboModule,
                const facebook::jsi::Value args[], size_t count);

  static facebook::jsi::Value
  JSI__booleanMethod(facebook::jsi::Runtime &rt,
                facebook::react::TurboModule &turboModule,
                const facebook::jsi::Value args[], size_t count);

  static facebook::jsi::Value
  JSI__stringMethod(facebook::jsi::Runtime &rt,
                facebook::react::TurboModule &turboModule,
                const facebook::jsi::Value args[], size_t count);

  static facebook::jsi::Value
  JSI__objectMethod(facebook::jsi::Runtime &rt,
                facebook::react::TurboModule &turboModule,
                const facebook::jsi::Value args[], size_t count);

protected:
  std::shared_ptr<facebook::react::CallInvoker> callInvoker_;
};

} // namespace crabytest
} // namespace craby
