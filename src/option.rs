use std::{
    hint, mem,
    ops::{Deref, DerefMut},
    pin::Pin,
};

/// cbindgen:derive-tagged-enum-destructor
#[repr(C, u8)]
pub enum Option<T> {
    None,
    Some(T),
}

impl<T> From<std::option::Option<T>> for Option<T> {
    #[inline]
    fn from(value: std::option::Option<T>) -> Self {
        match value {
            None => Option::None,
            Some(val) => Option::Some(val),
        }
    }
}

impl<T> Into<std::option::Option<T>> for Option<T> {
    #[inline]
    fn into(self) -> std::option::Option<T> {
        match self {
            Option::None => None,
            Option::Some(val) => Some(val),
        }
    }
}

impl<T> Option<T> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the option is a [`Some`] value.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Option<u32> = Some(2);
    /// assert_eq!(x.is_some(), true);
    ///
    /// let x: Option<u32> = None;
    /// assert_eq!(x.is_some(), false);
    /// ```
    #[must_use = "if you intended to assert that this has a value, consider `.unwrap()` instead"]
    #[inline]
    pub const fn is_some(&self) -> bool {
        matches!(*self, Option::Some(_))
    }

    /// Returns `true` if the option is a [`Some`] and the value inside of it matches a predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(is_some_and)]
    ///
    /// let x: Option<u32> = Some(2);
    /// assert_eq!(x.is_some_and(|x| x > 1), true);
    ///
    /// let x: Option<u32> = Some(0);
    /// assert_eq!(x.is_some_and(|x| x > 1), false);
    ///
    /// let x: Option<u32> = None;
    /// assert_eq!(x.is_some_and(|x| x > 1), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn is_some_and(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            Option::None => false,
            Option::Some(x) => f(x),
        }
    }

    /// Returns `true` if the option is a [`None`] value.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Option<u32> = Some(2);
    /// assert_eq!(x.is_none(), false);
    ///
    /// let x: Option<u32> = None;
    /// assert_eq!(x.is_none(), true);
    /// ```
    #[must_use = "if you intended to assert that this doesn't have a value, consider \
                  `.and_then(|_| panic!(\"`Option` had a value when expected `None`\"))` instead"]
    #[inline]
    pub const fn is_none(&self) -> bool {
        !self.is_some()
    }

    /////////////////////////////////////////////////////////////////////////
    // Adapter for working with references
    /////////////////////////////////////////////////////////////////////////

    /// Converts from `&Option<T>` to `Option<&T>`.
    ///
    /// # Examples
    ///
    /// Calculates the length of an <code>Option<[String]></code> as an <code>Option<[usize]></code>
    /// without moving the [`String`]. The [`map`] method takes the `self` argument by value,
    /// consuming the original, so this technique uses `as_ref` to first take an `Option` to a
    /// reference to the value inside the original.
    ///
    /// [`map`]: Option::map
    /// [String]: ../../std/string/struct.String.html "String"
    /// [`String`]: ../../std/string/struct.String.html "String"
    ///
    /// ```
    /// let text: Option<String> = Some("Hello, world!".to_string());
    /// // First, cast `Option<String>` to `Option<&String>` with `as_ref`,
    /// // then consume *that* with `map`, leaving `text` on the stack.
    /// let text_length: Option<usize> = text.as_ref().map(|s| s.len());
    /// println!("still can print text: {text:?}");
    /// ```
    #[inline]
    pub const fn as_ref(&self) -> std::option::Option<&T> {
        match *self {
            Option::Some(ref x) => Some(x),
            Option::None => None,
        }
    }

    /// Converts from `&mut Option<T>` to `Option<&mut T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = Some(2);
    /// match x.as_mut() {
    ///     Some(v) => *v = 42,
    ///     None => {},
    /// }
    /// assert_eq!(x, Some(42));
    /// ```
    #[inline]
    pub fn as_mut(&mut self) -> std::option::Option<&mut T> {
        match *self {
            Option::Some(ref mut x) => Some(x),
            Option::None => None,
        }
    }

    /// Converts from <code>[Pin]<[&]Option\<T>></code> to <code>Option<[Pin]<[&]T>></code>.
    ///
    /// [&]: reference "shared reference"
    #[inline]
    #[must_use]
    pub fn as_pin_ref(self: Pin<&Self>) -> std::option::Option<Pin<&T>> {
        match Pin::get_ref(self).as_ref() {
            // SAFETY: `x` is guaranteed to be pinned because it comes from `self`
            // which is pinned.
            Some(x) => unsafe { Some(Pin::new_unchecked(x)) },
            None => None,
        }
    }

    /// Returns the contained [`Some`] value, consuming the `self` value.
    ///
    /// Because this function may panic, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the [`None`]
    /// case explicitly, or call [`unwrap_or`], [`unwrap_or_else`], or
    /// [`unwrap_or_default`].
    ///
    /// [`unwrap_or`]: Option::unwrap_or
    /// [`unwrap_or_else`]: Option::unwrap_or_else
    /// [`unwrap_or_default`]: Option::unwrap_or_default
    ///
    /// # Panics
    ///
    /// Panics if the self value equals [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some("air");
    /// assert_eq!(x.unwrap(), "air");
    /// ```
    ///
    /// ```should_panic
    /// let x: Option<&str> = None;
    /// assert_eq!(x.unwrap(), "air"); // fails
    /// ```
    #[inline]
    #[track_caller]
    pub fn unwrap(self) -> T {
        match self {
            Option::Some(val) => val,
            Option::None => panic!("called `Option::unwrap()` on a `None` value"),
        }
    }

    /// Returns the contained [`Some`] value or a provided default.
    ///
    /// Arguments passed to `unwrap_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`unwrap_or_else`],
    /// which is lazily evaluated.
    ///
    /// [`unwrap_or_else`]: Option::unwrap_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(Some("car").unwrap_or("bike"), "car");
    /// assert_eq!(None.unwrap_or("bike"), "bike");
    /// ```
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Option::Some(x) => x,
            Option::None => default,
        }
    }

    /// Returns the contained [`Some`] value or computes it from a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// let k = 10;
    /// assert_eq!(Some(4).unwrap_or_else(|| 2 * k), 4);
    /// assert_eq!(None.unwrap_or_else(|| 2 * k), 20);
    /// ```
    #[inline]
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Option::Some(x) => x,
            Option::None => f(),
        }
    }

    /// Returns the contained [`Some`] value or a default.
    ///
    /// Consumes the `self` argument then, if [`Some`], returns the contained
    /// value, otherwise if [`None`], returns the [default value] for that
    /// type.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Option<u32> = None;
    /// let y: Option<u32> = Some(12);
    ///
    /// assert_eq!(x.unwrap_or_default(), 0);
    /// assert_eq!(y.unwrap_or_default(), 12);
    /// ```
    ///
    /// [default value]: Default::default
    /// [`parse`]: str::parse
    /// [`FromStr`]: crate::str::FromStr
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Option::Some(x) => x,
            Option::None => Default::default(),
        }
    }

    /// Returns the contained [`Some`] value, consuming the `self` value,
    /// without checking that the value is not [`None`].
    ///
    /// # Safety
    ///
    /// Calling this method on [`None`] is *[undefined behavior]*.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some("air");
    /// assert_eq!(unsafe { x.unwrap_unchecked() }, "air");
    /// ```
    ///
    /// ```no_run
    /// let x: Option<&str> = None;
    /// assert_eq!(unsafe { x.unwrap_unchecked() }, "air"); // Undefined behavior!
    /// ```
    #[inline]
    #[track_caller]
    pub unsafe fn unwrap_unchecked(self) -> T {
        debug_assert!(self.is_some());
        match self {
            Option::Some(val) => val,
            // SAFETY: the safety contract must be upheld by the caller.
            Option::None => unsafe { hint::unreachable_unchecked() },
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Transforming contained values
    /////////////////////////////////////////////////////////////////////////

    /// Maps an `Option<T>` to `Option<U>` by applying a function to a contained value (if `Some`) or returns `None` (if `None`).
    ///
    /// # Examples
    ///
    /// Calculates the length of an <code>Option<[String]></code> as an
    /// <code>Option<[usize]></code>, consuming the original:
    ///
    /// [String]: ../../std/string/struct.String.html "String"
    /// ```
    /// let maybe_some_string = Some(String::from("Hello, World!"));
    /// // `Option::map` takes self *by value*, consuming `maybe_some_string`
    /// let maybe_some_len = maybe_some_string.map(|s| s.len());
    /// assert_eq!(maybe_some_len, Some(13));
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.map(|s| s.len()), None);
    /// ```
    #[inline]
    pub fn map<U, F>(self, f: F) -> std::option::Option<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Option::Some(x) => Some(f(x)),
            Option::None => None,
        }
    }

    /// Calls the provided closure with a reference to the contained value (if [`Some`]).
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(result_option_inspect)]
    ///
    /// let v = vec![1, 2, 3, 4, 5];
    ///
    /// // prints "got: 4"
    /// let x: Option<&usize> = v.get(3).inspect(|x| println!("got: {x}"));
    ///
    /// // prints nothing
    /// let x: Option<&usize> = v.get(5).inspect(|x| println!("got: {x}"));
    /// ```
    #[inline]
    pub fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Option::Some(ref x) = self {
            f(x);
        }

        self
    }

    /// Returns the provided default result (if none),
    /// or applies a function to the contained value (if any).
    ///
    /// Arguments passed to `map_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`map_or_else`],
    /// which is lazily evaluated.
    ///
    /// [`map_or_else`]: Option::map_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some("foo");
    /// assert_eq!(x.map_or(42, |v| v.len()), 3);
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.map_or(42, |v| v.len()), 42);
    /// ```
    #[inline]
    pub fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Option::Some(t) => f(t),
            Option::None => default,
        }
    }

    /// Computes a default function result (if none), or
    /// applies a different function to the contained value (if any).
    ///
    /// # Examples
    ///
    /// ```
    /// let k = 21;
    ///
    /// let x = Some("foo");
    /// assert_eq!(x.map_or_else(|| 2 * k, |v| v.len()), 3);
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.map_or_else(|| 2 * k, |v| v.len()), 42);
    /// ```
    #[inline]
    pub fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        match self {
            Option::Some(t) => f(t),
            Option::None => default(),
        }
    }

    /// Transforms the `Option<T>` into a [`Result<T, E>`], mapping [`Some(v)`] to
    /// [`Ok(v)`] and [`None`] to [`Err(err)`].
    ///
    /// Arguments passed to `ok_or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`ok_or_else`], which is
    /// lazily evaluated.
    ///
    /// [`Ok(v)`]: Ok
    /// [`Err(err)`]: Err
    /// [`Some(v)`]: Some
    /// [`ok_or_else`]: Option::ok_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some("foo");
    /// assert_eq!(x.ok_or(0), Ok("foo"));
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.ok_or(0), Err(0));
    /// ```
    #[inline]
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Option::Some(v) => Ok(v),
            Option::None => Err(err),
        }
    }

    /// Transforms the `Option<T>` into a [`Result<T, E>`], mapping [`Some(v)`] to
    /// [`Ok(v)`] and [`None`] to [`Err(err())`].
    ///
    /// [`Ok(v)`]: Ok
    /// [`Err(err())`]: Err
    /// [`Some(v)`]: Some
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some("foo");
    /// assert_eq!(x.ok_or_else(|| 0), Ok("foo"));
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.ok_or_else(|| 0), Err(0));
    /// ```
    #[inline]
    pub fn ok_or_else<E, F>(self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E,
    {
        match self {
            Option::Some(v) => Ok(v),
            Option::None => Err(err()),
        }
    }

    /// Converts from `Option<T>` (or `&Option<T>`) to `Option<&T::Target>`.
    ///
    /// Leaves the original Option in-place, creating a new one with a reference
    /// to the original one, additionally coercing the contents via [`Deref`].
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Option<String> = Some("hey".to_owned());
    /// assert_eq!(x.as_deref(), Some("hey"));
    ///
    /// let x: Option<String> = None;
    /// assert_eq!(x.as_deref(), None);
    /// ```
    pub fn as_deref(&self) -> std::option::Option<&T::Target>
    where
        T: Deref,
    {
        match self.as_ref() {
            Some(t) => Some(t.deref()),
            None => None,
        }
    }

    /// Converts from `Option<T>` (or `&mut Option<T>`) to `Option<&mut T::Target>`.
    ///
    /// Leaves the original `Option` in-place, creating a new one containing a mutable reference to
    /// the inner type's [`Deref::Target`] type.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x: Option<String> = Some("hey".to_owned());
    /// assert_eq!(x.as_deref_mut().map(|x| {
    ///     x.make_ascii_uppercase();
    ///     x
    /// }), Some("HEY".to_owned().as_mut_str()));
    /// ```
    pub fn as_deref_mut(&mut self) -> std::option::Option<&mut T::Target>
    where
        T: DerefMut,
    {
        match self.as_mut() {
            Some(t) => Some(t.deref_mut()),
            None => None,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Boolean operations on the values, eager and lazy
    /////////////////////////////////////////////////////////////////////////

    /// Returns [`None`] if the option is [`None`], otherwise returns `optb`.
    ///
    /// Arguments passed to `and` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`and_then`], which is
    /// lazily evaluated.
    ///
    /// [`and_then`]: Option::and_then
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some(2);
    /// let y: Option<&str> = None;
    /// assert_eq!(x.and(y), None);
    ///
    /// let x: Option<u32> = None;
    /// let y = Some("foo");
    /// assert_eq!(x.and(y), None);
    ///
    /// let x = Some(2);
    /// let y = Some("foo");
    /// assert_eq!(x.and(y), Some("foo"));
    ///
    /// let x: Option<u32> = None;
    /// let y: Option<&str> = None;
    /// assert_eq!(x.and(y), None);
    /// ```
    #[inline]
    pub fn and<U>(self, optb: std::option::Option<U>) -> std::option::Option<U> {
        match self {
            Option::Some(_) => optb,
            Option::None => None,
        }
    }

    /// Returns [`None`] if the option is [`None`], otherwise calls `f` with the
    /// wrapped value and returns the result.
    ///
    /// Some languages call this operation flatmap.
    ///
    /// # Examples
    ///
    /// ```
    /// fn sq_then_to_string(x: u32) -> Option<String> {
    ///     x.checked_mul(x).map(|sq| sq.to_string())
    /// }
    ///
    /// assert_eq!(Some(2).and_then(sq_then_to_string), Some(4.to_string()));
    /// assert_eq!(Some(1_000_000).and_then(sq_then_to_string), None); // overflowed!
    /// assert_eq!(None.and_then(sq_then_to_string), None);
    /// ```
    ///
    /// Often used to chain fallible operations that may return [`None`].
    ///
    /// ```
    /// let arr_2d = [["A0", "A1"], ["B0", "B1"]];
    ///
    /// let item_0_1 = arr_2d.get(0).and_then(|row| row.get(1));
    /// assert_eq!(item_0_1, Some(&"A1"));
    ///
    /// let item_2_0 = arr_2d.get(2).and_then(|row| row.get(0));
    /// assert_eq!(item_2_0, None);
    /// ```
    #[inline]
    pub fn and_then<U, F>(self, f: F) -> std::option::Option<U>
    where
        F: FnOnce(T) -> std::option::Option<U>,
    {
        match self {
            Option::Some(x) => f(x),
            Option::None => None,
        }
    }

    /// Returns [`None`] if the option is [`None`], otherwise calls `predicate`
    /// with the wrapped value and returns:
    ///
    /// - [`Some(t)`] if `predicate` returns `true` (where `t` is the wrapped
    ///   value), and
    /// - [`None`] if `predicate` returns `false`.
    ///
    /// This function works similar to [`Iterator::filter()`]. You can imagine
    /// the `Option<T>` being an iterator over one or zero elements. `filter()`
    /// lets you decide which elements to keep.
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn is_even(n: &i32) -> bool {
    ///     n % 2 == 0
    /// }
    ///
    /// assert_eq!(None.filter(is_even), None);
    /// assert_eq!(Some(3).filter(is_even), None);
    /// assert_eq!(Some(4).filter(is_even), Some(4));
    /// ```
    ///
    /// [`Some(t)`]: Some
    #[inline]
    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: FnOnce(&T) -> bool,
    {
        if let Option::Some(x) = self {
            if predicate(&x) {
                return Option::Some(x);
            }
        }
        Option::None
    }

    /// Returns the option if it contains a value, otherwise returns `optb`.
    ///
    /// Arguments passed to `or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`or_else`], which is
    /// lazily evaluated.
    ///
    /// [`or_else`]: Option::or_else
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some(2);
    /// let y = None;
    /// assert_eq!(x.or(y), Some(2));
    ///
    /// let x = None;
    /// let y = Some(100);
    /// assert_eq!(x.or(y), Some(100));
    ///
    /// let x = Some(2);
    /// let y = Some(100);
    /// assert_eq!(x.or(y), Some(2));
    ///
    /// let x: Option<u32> = None;
    /// let y = None;
    /// assert_eq!(x.or(y), None);
    /// ```
    #[inline]
    pub fn or(self, optb: std::option::Option<T>) -> std::option::Option<T> {
        match self {
            Option::Some(x) => Some(x),
            Option::None => optb,
        }
    }

    /// Returns the option if it contains a value, otherwise calls `f` and
    /// returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// fn nobody() -> Option<&'static str> { None }
    /// fn vikings() -> Option<&'static str> { Some("vikings") }
    ///
    /// assert_eq!(Some("barbarians").or_else(vikings), Some("barbarians"));
    /// assert_eq!(None.or_else(vikings), Some("vikings"));
    /// assert_eq!(None.or_else(nobody), None);
    /// ```
    #[inline]
    pub fn or_else<F>(self, f: F) -> std::option::Option<T>
    where
        F: FnOnce() -> std::option::Option<T>,
    {
        match self {
            Option::Some(x) => Some(x),
            Option::None => f(),
        }
    }

    /// Returns [`Some`] if exactly one of `self`, `optb` is [`Some`], otherwise returns [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some(2);
    /// let y: Option<u32> = None;
    /// assert_eq!(x.xor(y), Some(2));
    ///
    /// let x: Option<u32> = None;
    /// let y = Some(2);
    /// assert_eq!(x.xor(y), Some(2));
    ///
    /// let x = Some(2);
    /// let y = Some(2);
    /// assert_eq!(x.xor(y), None);
    ///
    /// let x: Option<u32> = None;
    /// let y: Option<u32> = None;
    /// assert_eq!(x.xor(y), None);
    /// ```
    #[inline]
    pub fn xor(self, optb: Option<T>) -> std::option::Option<T> {
        match (self, optb) {
            (Option::Some(a), Option::None) => Some(a),
            (Option::None, Option::Some(b)) => Some(b),
            _ => None,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Entry-like operations to insert a value and return a reference
    /////////////////////////////////////////////////////////////////////////

    /// Inserts `value` into the option, then returns a mutable reference to it.
    ///
    /// If the option already contains a value, the old value is dropped.
    ///
    /// See also [`Option::get_or_insert`], which doesn't update the value if
    /// the option already contains [`Some`].
    ///
    /// # Example
    ///
    /// ```
    /// let mut opt = None;
    /// let val = opt.insert(1);
    /// assert_eq!(*val, 1);
    /// assert_eq!(opt.unwrap(), 1);
    /// let val = opt.insert(2);
    /// assert_eq!(*val, 2);
    /// *val = 3;
    /// assert_eq!(opt.unwrap(), 3);
    /// ```
    #[must_use = "if you intended to set a value, consider assignment instead"]
    #[inline]
    pub fn insert(&mut self, value: T) -> &mut T {
        *self = Option::Some(value);

        // SAFETY: the code above just filled the option
        unsafe { self.as_mut().unwrap_unchecked() }
    }

    /// Inserts `value` into the option if it is [`None`], then
    /// returns a mutable reference to the contained value.
    ///
    /// See also [`Option::insert`], which updates the value even if
    /// the option already contains [`Some`].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = None;
    ///
    /// {
    ///     let y: &mut u32 = x.get_or_insert(5);
    ///     assert_eq!(y, &5);
    ///
    ///     *y = 7;
    /// }
    ///
    /// assert_eq!(x, Some(7));
    /// ```
    #[inline]
    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        if let Option::None = *self {
            *self = Option::Some(value);
        }

        // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
        // variant in the code above.
        unsafe { self.as_mut().unwrap_unchecked() }
    }

    /// Inserts a value computed from `f` into the option if it is [`None`],
    /// then returns a mutable reference to the contained value.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = None;
    ///
    /// {
    ///     let y: &mut u32 = x.get_or_insert_with(|| 5);
    ///     assert_eq!(y, &5);
    ///
    ///     *y = 7;
    /// }
    ///
    /// assert_eq!(x, Some(7));
    /// ```
    #[inline]
    pub fn get_or_insert_with<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        if let Option::None = *self {
            // the compiler isn't smart enough to know that we are not dropping a `T`
            // here and wants us to ensure `T` can be dropped at compile time.
            mem::forget(mem::replace(self, Option::Some(f())))
        }

        // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
        // variant in the code above.
        unsafe { self.as_mut().unwrap_unchecked() }
    }

    /////////////////////////////////////////////////////////////////////////
    // Misc
    /////////////////////////////////////////////////////////////////////////

    /// Takes the value out of the option, leaving a [`None`] in its place.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = Some(2);
    /// let y = x.take();
    /// assert_eq!(x, None);
    /// assert_eq!(y, Some(2));
    ///
    /// let mut x: Option<u32> = None;
    /// let y = x.take();
    /// assert_eq!(x, None);
    /// assert_eq!(y, None);
    /// ```
    #[inline]
    pub fn take(&mut self) -> Option<T> {
        // FIXME replace `mem::replace` by `mem::take` when the latter is const ready
        mem::replace(self, Option::None)
    }

    /// Replaces the actual value in the option by the value given in parameter,
    /// returning the old value if present,
    /// leaving a [`Some`] in its place without deinitializing either one.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = Some(2);
    /// let old = x.replace(5);
    /// assert_eq!(x, Some(5));
    /// assert_eq!(old, Some(2));
    ///
    /// let mut x = None;
    /// let old = x.replace(3);
    /// assert_eq!(x, Some(3));
    /// assert_eq!(old, None);
    /// ```
    #[inline]
    pub fn replace(&mut self, value: T) -> Option<T> {
        mem::replace(self, Option::Some(value))
    }

    /// Returns `true` if the option is a [`Some`] value containing the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(option_result_contains)]
    ///
    /// let x: Option<u32> = Some(2);
    /// assert_eq!(x.contains(&2), true);
    ///
    /// let x: Option<u32> = Some(3);
    /// assert_eq!(x.contains(&2), false);
    ///
    /// let x: Option<u32> = None;
    /// assert_eq!(x.contains(&2), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn contains<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            Option::Some(y) => x.eq(y),
            Option::None => false,
        }
    }

    /// Zips `self` with another `Option`.
    ///
    /// If `self` is `Some(s)` and `other` is `Some(o)`, this method returns `Some((s, o))`.
    /// Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some(1);
    /// let y = Some("hi");
    /// let z = None::<u8>;
    ///
    /// assert_eq!(x.zip(y), Some((1, "hi")));
    /// assert_eq!(x.zip(z), None);
    /// ```
    pub fn zip<U>(self, other: Option<U>) -> std::option::Option<(T, U)> {
        match (self, other) {
            (Option::Some(a), Option::Some(b)) => Some((a, b)),
            _ => None,
        }
    }

    /// Zips `self` and another `Option` with function `f`.
    ///
    /// If `self` is `Some(s)` and `other` is `Some(o)`, this method returns `Some(f(s, o))`.
    /// Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(option_zip)]
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Point {
    ///     x: f64,
    ///     y: f64,
    /// }
    ///
    /// impl Point {
    ///     fn new(x: f64, y: f64) -> Self {
    ///         Self { x, y }
    ///     }
    /// }
    ///
    /// let x = Some(17.5);
    /// let y = Some(42.7);
    ///
    /// assert_eq!(x.zip_with(y, Point::new), Some(Point { x: 17.5, y: 42.7 }));
    /// assert_eq!(x.zip_with(None, Point::new), None);
    /// ```
    pub fn zip_with<U, F, R>(self, other: Option<U>, f: F) -> std::option::Option<R>
    where
        F: FnOnce(T, U) -> R,
    {
        match (self, other) {
            (Option::Some(a), Option::Some(b)) => Some(f(a, b)),
            _ => None,
        }
    }
}

impl<T, U> Option<(T, U)> {
    /// Unzips an option containing a tuple of two options.
    ///
    /// If `self` is `Some((a, b))` this method returns `(Some(a), Some(b))`.
    /// Otherwise, `(None, None)` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some((1, "hi"));
    /// let y = None::<(u8, u32)>;
    ///
    /// assert_eq!(x.unzip(), (Some(1), Some("hi")));
    /// assert_eq!(y.unzip(), (None, None));
    /// ```
    #[inline]
    pub fn unzip(self) -> (std::option::Option<T>, std::option::Option<U>) {
        match self {
            Option::Some((a, b)) => (Some(a), Some(b)),
            Option::None => (None, None),
        }
    }
}

impl<T> Option<&T> {
    /// Maps an `Option<&T>` to an `Option<T>` by copying the contents of the
    /// option.
    ///
    /// # Examples
    ///
    /// ```
    /// let x = 12;
    /// let opt_x = Some(&x);
    /// assert_eq!(opt_x, Some(&12));
    /// let copied = opt_x.copied();
    /// assert_eq!(copied, Some(12));
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub const fn copied(self) -> std::option::Option<T>
    where
        T: Copy,
    {
        // FIXME: this implementation, which sidesteps using `Option::map` since it's not const
        // ready yet, should be reverted when possible to avoid code repetition
        match self {
            Option::Some(&v) => Some(v),
            Option::None => None,
        }
    }

    /// Maps an `Option<&T>` to an `Option<T>` by cloning the contents of the
    /// option.
    ///
    /// # Examples
    ///
    /// ```
    /// let x = 12;
    /// let opt_x = Some(&x);
    /// assert_eq!(opt_x, Some(&12));
    /// let cloned = opt_x.cloned();
    /// assert_eq!(cloned, Some(12));
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn cloned(self) -> std::option::Option<T>
    where
        T: Clone,
    {
        match self {
            Option::Some(t) => Some(t.clone()),
            Option::None => None,
        }
    }
}

impl<T> Option<&mut T> {
    /// Maps an `Option<&mut T>` to an `Option<T>` by copying the contents of the
    /// option.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = 12;
    /// let opt_x = Some(&mut x);
    /// assert_eq!(opt_x, Some(&mut 12));
    /// let copied = opt_x.copied();
    /// assert_eq!(copied, Some(12));
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn copied(self) -> std::option::Option<T>
    where
        T: Copy,
    {
        match self {
            Option::Some(&mut t) => Some(t),
            Option::None => None,
        }
    }

    /// Maps an `Option<&mut T>` to an `Option<T>` by cloning the contents of the
    /// option.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = 12;
    /// let opt_x = Some(&mut x);
    /// assert_eq!(opt_x, Some(&mut 12));
    /// let cloned = opt_x.cloned();
    /// assert_eq!(cloned, Some(12));
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn cloned(self) -> std::option::Option<T>
    where
        T: Clone,
    {
        match self {
            Option::Some(t) => Some(t.clone()),
            Option::None => None,
        }
    }
}

impl<T, E> Option<Result<T, E>> {
    /// Transposes an `Option` of a [`Result`] into a [`Result`] of an `Option`.
    ///
    /// [`None`] will be mapped to <code>[Ok]\([None])</code>.
    /// <code>[Some]\([Ok]\(\_))</code> and <code>[Some]\([Err]\(\_))</code> will be mapped to
    /// <code>[Ok]\([Some]\(\_))</code> and <code>[Err]\(\_)</code>.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Eq, PartialEq)]
    /// struct SomeErr;
    ///
    /// let x: Result<Option<i32>, SomeErr> = Ok(Some(5));
    /// let y: Option<Result<i32, SomeErr>> = Some(Ok(5));
    /// assert_eq!(x, y.transpose());
    /// ```
    #[inline]
    pub fn transpose(self) -> Result<std::option::Option<T>, E> {
        match self {
            Option::Some(Ok(x)) => Ok(Some(x)),
            Option::Some(Err(e)) => Err(e),
            Option::None => Ok(None),
        }
    }
}

impl<T> Clone for Option<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Option::Some(x) => Option::Some(x.clone()),
            Option::None => Option::None,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Option::Some(to), Option::Some(from)) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}
