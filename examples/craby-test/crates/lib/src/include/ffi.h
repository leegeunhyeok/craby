#pragma once
#include "rust/cxx.h"

namespace craby {
namespace ffi {
namespace crabytest {

  struct JSNumber;
  struct JSBoolean;
  struct JSString;

  JSNumber numericModule(JSNumber arg) const;
  JSBoolean booleanModule(JSBoolean arg) const;
  JSString stringModule(JSString arg) const;

} // namespace crabytest
} // namespace ffi
} // namespace craby
