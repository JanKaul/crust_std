#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

namespace crust_std {

template<typename T>
struct ArcInner {
  AtomicUsize rc;
  T data;
};

template<typename T>
struct Arc {
  ArcInner<T> *ptr;
};

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
};

template<typename T>
struct RawVec {
  T *ptr;
  uintptr_t cap;
};

template<typename T>
struct Vec {
  RawVec<T> buf;
  uintptr_t len;
};

struct String {
  Vec<uint8_t> _0;
};

extern "C" {

const T *arc_get(const Arc<T> *arc);

Arc<T> arc_clone(const Arc<T> *arc);

void arc_free(Arc<T> arc);

bool option_has_value(const Option<T> *option);

const T *option_value(const Option<T> *option);

uintptr_t string_len(const String *string);

const uint8_t *string_at(const String *string, uintptr_t i);

const uint8_t *string_data(const String *string);

uintptr_t vec_len(const Vec<T> *vec);

const T *vec_at(const Vec<T> *vec, uintptr_t i);

const T *vec_data(const Vec<T> *vec);

} // extern "C"

} // namespace crust_std
