#include "NativeCalculatorCpp.h"

namespace facebook::react {

NativeCalculatorCpp::NativeCalculatorCpp(std::shared_ptr<CallInvoker> jsInvoker)
    : NativeCalculatorCppCxxSpec(std::move(jsInvoker)) {}

std::string NativeSampleModule::reverseString(jsi::Runtime& rt, std::string input) {
  return std::string(input.rbegin(), input.rend());
}

} // namespace facebook::react
