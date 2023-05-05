typedef void Void;
template<typename T>
using Opaque=void;
typedef uintptr_t AtomicUsize;

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

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

uintptr_t crust_string_len(const String *string);

const uint8_t *crust_string_at(const String *string, uintptr_t i);

const uint8_t *crust_string_data(const String *string);

void crust_free_vec_u8(Vec<uint8_t> *vec);

} // extern "C"
