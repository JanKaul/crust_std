#pragma once

#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>
#include <span>


namespace iceberg {

template<typename T>
struct Option {
  enum class Tag : uint8_t {
    None,
    Some,
  };

  struct Some_Body {
    T _0;
  };

  Tag tag;
  union {
    Some_Body some;
  };

  ~Option() {
    switch (tag) {
      case Tag::Some: some.~Some_Body(); break;
      default: break;
    }
  }

  Option(const Option& other)
   : tag(other.tag) {
    switch (tag) {
      case Tag::Some: ::new (&some) (Some_Body)(other.some); break;
      default: break;
    }
  }
};

/// A struct that basically replaces a `Box<[T]>`, but which cbindgen can
/// understand.
///
/// We could rely on the struct layout of `Box<[T]>` per:
///
///   https://github.com/rust-lang/unsafe-code-guidelines/blob/master/reference/src/layout/pointers.md
///
/// But handling fat pointers with cbindgen both in structs and argument
/// positions more generally is a bit tricky.
///
template<typename T>
struct OwnedSlice {
  T *ptr;
  size_t len;
  std::span<T> AsSpan() {
    return { ptr, len };
  }

  inline std::span<const T> AsSpan() const {
    return { ptr, len };
  }
  
  ~OwnedSlice() {
    if (!len)
      return;
    for (auto& val : AsSpan())
      val.~T();
    free(ptr);
    ptr = (T*)alignof(T);
    len = 0; 
  }
};

/// A struct that basically replaces a Box<str>, but with a defined layout,
/// suitable for FFI.
struct OwnedStr {
  OwnedSlice<uint8_t> _0;
};

} // namespace iceberg
