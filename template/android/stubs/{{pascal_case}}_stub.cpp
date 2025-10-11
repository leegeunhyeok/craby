#include "{{ pascal_case }}_stub.h"

namespace facebook::react {

std::shared_ptr<TurboModule> {{ pascal_case }}_stub_ModuleProvider(const std::string &moduleName, const JavaTurboModule::InitParams &params) {
  return nullptr;
}

} // namespace facebook::react
