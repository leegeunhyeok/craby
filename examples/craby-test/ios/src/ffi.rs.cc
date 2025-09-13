#include <array>
#include <cstddef>
#include <cstdint>
#include <new>
#include <string>
#include <type_traits>
#include <utility>

#ifdef __GNUC__
#pragma GCC diagnostic ignored "-Wshadow"
#ifdef __clang__
#pragma clang diagnostic ignored "-Wdollar-in-identifier-extension"
#endif // __clang__
#endif // __GNUC__

namespace rust {
inline namespace cxxbridge1 {
// #include "rust/cxx.h"

struct unsafe_bitcopy_t;

#ifndef CXXBRIDGE1_RUST_STRING
#define CXXBRIDGE1_RUST_STRING
class String final {
public:
  String() noexcept;
  String(const String &) noexcept;
  String(String &&) noexcept;
  ~String() noexcept;

  String(const std::string &);
  String(const char *);
  String(const char *, std::size_t);
  String(const char16_t *);
  String(const char16_t *, std::size_t);
#ifdef __cpp_char8_t
  String(const char8_t *s);
  String(const char8_t *s, std::size_t len);
#endif

  static String lossy(const std::string &) noexcept;
  static String lossy(const char *) noexcept;
  static String lossy(const char *, std::size_t) noexcept;
  static String lossy(const char16_t *) noexcept;
  static String lossy(const char16_t *, std::size_t) noexcept;

  String &operator=(const String &) & noexcept;
  String &operator=(String &&) & noexcept;

  explicit operator std::string() const;

  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  const char *c_str() noexcept;

  std::size_t capacity() const noexcept;
  void reserve(size_t new_cap) noexcept;

  using iterator = char *;
  iterator begin() noexcept;
  iterator end() noexcept;

  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const String &) const noexcept;
  bool operator!=(const String &) const noexcept;
  bool operator<(const String &) const noexcept;
  bool operator<=(const String &) const noexcept;
  bool operator>(const String &) const noexcept;
  bool operator>=(const String &) const noexcept;

  void swap(String &) noexcept;

  String(unsafe_bitcopy_t, const String &) noexcept;

private:
  struct lossy_t;
  String(lossy_t, const char *, std::size_t) noexcept;
  String(lossy_t, const char16_t *, std::size_t) noexcept;
  friend void swap(String &lhs, String &rhs) noexcept { lhs.swap(rhs); }

  std::array<std::uintptr_t, 3> repr;
};
#endif // CXXBRIDGE1_RUST_STRING

namespace detail {
template <typename T, typename = void *>
struct operator_new {
  void *operator()(::std::size_t sz) { return ::operator new(sz); }
};

template <typename T>
struct operator_new<T, decltype(T::operator new(sizeof(T)))> {
  void *operator()(::std::size_t sz) { return T::operator new(sz); }
};
} // namespace detail

template <typename T>
union ManuallyDrop {
  T value;
  ManuallyDrop(T &&value) : value(::std::move(value)) {}
  ~ManuallyDrop() {}
};

template <typename T>
union MaybeUninit {
  T value;
  void *operator new(::std::size_t sz) { return detail::operator_new<T>{}(sz); }
  MaybeUninit() {}
  ~MaybeUninit() {}
};
} // namespace cxxbridge1
} // namespace rust

#if __cplusplus >= 201402L
#define CXX_DEFAULT_VALUE(value) = value
#else
#define CXX_DEFAULT_VALUE(value)
#endif

namespace craby {
  namespace codegen {
    namespace crabytest {
      struct TestObject;
    }
  }
}

namespace craby {
namespace codegen {
namespace crabytest {
#ifndef CXXBRIDGE1_STRUCT_craby$codegen$crabytest$TestObject
#define CXXBRIDGE1_STRUCT_craby$codegen$crabytest$TestObject
struct TestObject final {
  ::rust::String foo;
  double bar CXX_DEFAULT_VALUE(0);
  bool baz CXX_DEFAULT_VALUE(false);

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_craby$codegen$crabytest$TestObject

extern "C" {
double craby$codegen$crabytest$cxxbridge1$numeric_method(double arg) noexcept;

bool craby$codegen$crabytest$cxxbridge1$boolean_method(bool arg) noexcept;

void craby$codegen$crabytest$cxxbridge1$string_method(::rust::String *arg, ::rust::String *return$) noexcept;

void craby$codegen$crabytest$cxxbridge1$object_method(::craby::codegen::crabytest::TestObject *arg, ::craby::codegen::crabytest::TestObject *return$) noexcept;
} // extern "C"

double numericMethod(double arg) noexcept {
  return craby$codegen$crabytest$cxxbridge1$numeric_method(arg);
}

bool booleanMethod(bool arg) noexcept {
  return craby$codegen$crabytest$cxxbridge1$boolean_method(arg);
}

::rust::String stringMethod(::rust::String arg) noexcept {
  ::rust::MaybeUninit<::rust::String> return$;
  craby$codegen$crabytest$cxxbridge1$string_method(&arg, &return$.value);
  return ::std::move(return$.value);
}

::craby::codegen::crabytest::TestObject objectMethod(::craby::codegen::crabytest::TestObject arg) noexcept {
  ::rust::ManuallyDrop<::craby::codegen::crabytest::TestObject> arg$(::std::move(arg));
  ::rust::MaybeUninit<::craby::codegen::crabytest::TestObject> return$;
  craby$codegen$crabytest$cxxbridge1$object_method(&arg$.value, &return$.value);
  return ::std::move(return$.value);
}
} // namespace crabytest
} // namespace codegen
} // namespace craby
