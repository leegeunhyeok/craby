#pragma once

#include <CalculatorCppSpecsJSI.h>

#include <memory>
#include <string>

namespace facebook::react {

class CalculatorCpp : public CalculatorCppCxxSpec<CalculatorCpp> {
public:
  CalculatorCpp(std::shared_ptr<CallInvoker> jsInvoker);

  std::string reverseString(jsi::Runtime& rt, std::string input);
};

} // namespace facebook::react
