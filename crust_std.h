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
struct ArcInner {
  AtomicUsize rc;
  T data;
};

template<typename T>
struct Arc {
  ArcInner<T> *ptr;
};

template<typename T>
using Opaque = Arc<T>;

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

const Void *crust_arc_get(const Opaque<Void> *arc);

Arc<Void> crust_arc_clone(const Opaque<Void> *arc);

void crust_arc_free(Opaque<Void> *arc);

bool crust_option_has_value(const Opaque<Void> *option);

const Void *crust_option_value(const Opaque<Void> *option);

uintptr_t crust_string_len(const String *string);

const uint8_t *crust_string_at(const String *string, uintptr_t i);

const uint8_t *crust_string_data(const String *string);

void crust_string_free(String *string);

uintptr_t crust_vec_len(const Opaque<Void> *vec);

const Void *crust_vec_at(const Opaque<Void> *vec, uintptr_t i);

const Void *crust_vec_data(const Opaque<Void> *vec);

void crust_vec_free(Opaque<Void> *vec);

} // extern "C"
