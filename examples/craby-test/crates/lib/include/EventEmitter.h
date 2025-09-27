#pragma once

#include "rust/cxx.h"
#include <functional>
#include <memory>
#include <unordered_map>

namespace craby {
namespace eventemitter {

using Delegate = std::function<void(const std::string& eventName)>;

class EventEmitterRegistry {
public:
  static EventEmitterRegistry& getInstance() {
    static EventEmitterRegistry instance;
    return instance;
  }

  void emit(uintptr_t id, rust::Str name) const {
    std::lock_guard<std::mutex> lock(mutex_);
    auto it = delegates_.find(id);
    if (it != delegates_.end()) {
      it->second(std::string(name));
    }
  }

  void registerDelegate(uintptr_t id, Delegate delegate) const {
    std::lock_guard<std::mutex> lock(mutex_);
    delegates_.insert_or_assign(id, delegate);
  }

  void unregisterDelegate(uintptr_t id) const {
    std::lock_guard<std::mutex> lock(mutex_);
    delegates_.erase(id);
  }

private:
  EventEmitterRegistry() = default;
  mutable std::unordered_map<uintptr_t, Delegate> delegates_;
  mutable std::mutex mutex_;
};

inline const EventEmitterRegistry& getEventEmitterRegistry() {
  return EventEmitterRegistry::getInstance();
}

} // namespace eventemitter
} // namespace craby
