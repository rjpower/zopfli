# Porting C to Rust -- Guidelines.

When asked to port a C module to Rust, follow these guidelines.

A skeleton Rust module has been created for you in the `rust` directory and
wired into CMake. You will need to adjust it as you add new Rust files or
replace the C module.

# Writing good CFFIs

* Unless otherwise specified, the C interface exposed from Rust must be identical to the original interface.
* You must port _every_ function in the module.
* The Rust implementation must, as closely as possible, be a one-to-one match to the C implementation.
  - Prefer to use similar function names and style - don't switch to idiomatic Rust without a clear reason.
* The Rust/C interface must be _safe_:
  - Don't return raw pointers to the C API, instead you must return a handle which is mapped to a Rust object.

Let's assume we have a C interface like:

```c
// foo.h
xmlFoo* xmlFooCreate(void);
void xmlFooFree(xmlFoo* foo);
void xmlFooPrint(xmlFoo* foo);
```

In Rust, we must expose the same CFFI:

```rust
// foo.rs
// Create a buffer
#[no_mangle]
pub extern "C" fn xmlFooCreate() -> xmlFooPtr
pub extern "C" fn xmlFooFree(foo: xmlFooPtr)
pub extern "C" fn xmlFooPrint(foo: xmlFooPtr)
```

But instead of returning a raw pointer, we must return a handle which is mapped to a Rust object. In this case we'll use a HashMap to map the C handles to Rust objects.

```rust
pub struct XmlFoo {}
pub type XmlFooPtr = usize;

static FOOS: OnceLock<Mutex<
    HashMap<XmlFooPtr, Box<XmlFoo>, BuildHasherDefault<DefaultHasher>>
>> = OnceLock::new();


#[no_mangle]
pub extern "C" fn xmlFooCreate() -> XmlFooPtr {
  let mutex = FOOS.get_or_init(|| {
      Mutex::new(HashMap::with_hasher(BuildHasherDefault::new()))
  });
  let mut m = mutex.lock().unwrap();
  let sz = m.len().try_into().unwrap();
  m.insert(sz, Box::new(XmlFoo {}));
  return sz;
}

#[no_mangle]
pub extern "C" fn xmlFooFree(foo: XmlFooPtr) {
  let mutex = FOOS.get_or_init(|| {
      Mutex::new(HashMap::with_hasher(BuildHasherDefault::new()))
  });
  let mut m = mutex.lock().unwrap();
  m.remove(&foo);
}

#[no_mangle]
pub extern "C" fn xmlFooPrint(foo: XmlFooPtr) {
  let mutex = FOOS.get_or_init(|| {
      Mutex::new(HashMap::with_hasher(BuildHasherDefault::new()))
  });
  let m = mutex.lock().unwrap();
  let foo = m.get(&foo).expect(&format!("foo not found at index {} was it freed?", foo));
  foo.print();
}
```

This works well for opaque C structures; if the C structure is not opaque, we
need a different strategy. If our API returns by value or fills a value, we can
of course simply fill the appropriate fields in our call:

```rust
#[repr(C)]
pub struct XmlBar {
  pub x: i32,
  pub y: i32,
  pub z: xmlFooPtr,
}

#[no_mangle]
pub extern "C" fn xmlBarCreate(x: i32, y: i32, z: xmlFooPtr) -> XmlBarPtr {
  XmlBar { x, y, z }
}

// or equivalently - this is unsafe, but it's the only way to create a struct with a pointer to a C object.
#[no_mangle]
pub extern "C" fn xmlBarCreate(bar: &mut XmlBar) -> XmlBarPtr {
  bar.z = xmlFooCreate();
  bar.x = 0;
  bar.y = 0;
  bar
}

```

If our API is not exposed externally, then we can change our API itself to be
opaque, and for example switch to using accessor functions to access individual
fields:

```rust
#[no_mangle]
pub extern "C" fn xmlBarGetX(bar: XmlBarPtr) -> i32 {
  let mutex = FOOS.get_or_init(|| {
      Mutex::new(HashMap::with_hasher(BuildHasherDefault::new()))
  });
  let m = mutex.lock().unwrap();
  let bar = m.get(&bar).expect(&format!("bar not found at index {} was it freed?", bar));
  bar.x
}
```

# Critical: Avoiding Memory Corruption from Struct Layout Mismatches

When defining Rust structs that correspond to C structs (especially when using
`#[repr(C)]`), you **MUST** ensure complete field compatibility. Incomplete or
incorrect struct definitions will cause silent memory corruption that can
manifest far from the actual bug location.

## Common Mistakes to Avoid

### Incomplete Struct Definitions

**WRONG** - This will cause memory corruption:
```rust
// Partial definition of xmlParserInput - DANGEROUS!
#[repr(C)]
pub struct XmlParserInput {
    pub base: *const XmlChar,
    pub cur: *const XmlChar,
    pub end: *const XmlChar,
    // Missing 14+ other fields!
}
```

**CORRECT** - Complete field definition:
```rust
// Complete definition matching C struct exactly
#[repr(C)]
pub struct XmlParserInput {
    pub buf: VoidPtr,
    pub filename: *const c_char,
    pub directory: *const c_char,
    pub base: *const XmlChar,
    pub cur: *const XmlChar,
    pub end: *const XmlChar,
    pub length: c_int,
    pub line: c_int,
    pub col: c_int,
    pub consumed: u64,
    pub free: VoidPtr,
    pub encoding: *const XmlChar,
    pub version: *const XmlChar,
    pub flags: c_int,
    pub id: c_int,
    pub parent_consumed: u64,
    pub entity: *mut c_void,
}
```

## Verification Steps

**Before defining any C struct in Rust:**

1. **Find the complete C definition** in the original header files
2. **Count all fields** - every single field must be present
3. **Verify field types** match exactly (sizes, signedness, pointer types)
4. **Check field order** - must match C declaration order exactly
5. **Verify struct size** using `std::mem::size_of::<YourStruct>()` vs C `sizeof(struct)`

## Why This Matters

- Incomplete structs cause **silent memory corruption**
- C code writing beyond the Rust struct boundary corrupts adjacent memory
- Symptoms appear far from the actual bug location (e.g., parser crashes from buffer corruption)
- Memory sanitizers may not detect these issues immediately
- The corruption can be intermittent and hard to reproduce

# Handling Conditional Compilation (`#ifdef` blocks)

C code often has complex conditional compilation. For Zopfli, check @port/CODEBASE_ANALYSIS.md which documents assumed active flags.

## Strategy: Include All Fields

**Always include ALL fields in Rust structs**, even those conditionally compiled in C:

```rust
// C struct with conditional compilation:
// typedef struct ZopfliHash {
//   int* head;
// #ifdef ZOPFLI_HASH_SAME_HASH  
//   int* head2;
// #endif
// #ifdef ZOPFLI_HASH_SAME
//   unsigned short* same;
// #endif
// } ZopfliHash;

// Rust equivalent - include ALL fields:
#[repr(C)]
pub struct ZopfliHashC {
    head: *mut c_int,
    head2: *mut c_int,     // Include even if conditionally compiled  
    same: *mut u16,        // Include even if conditionally compiled
}
```

**Why this works:**
- Zopfli assumes all optimization flags are enabled in production
- Including all fields ensures memory layout compatibility
- Simpler than conditional compilation in Rust

# Memory Management Patterns

## Replace Manual Allocation with Vec

**C Pattern:**
```c
void ZopfliAllocHash(size_t window_size, ZopfliHash* h) {
  h->head = (int*)malloc(sizeof(*h->head) * 65536);
  h->prev = (unsigned short*)malloc(sizeof(*h->prev) * window_size);
}

void ZopfliCleanHash(ZopfliHash* h) {
  free(h->head);
  free(h->prev);
}
```

**Rust Pattern:**
```rust
impl ZopfliHash {
    pub fn new(window_size: usize) -> Self {
        ZopfliHash {
            head: vec![-1; 65536],              // Initialize with default values
            prev: (0..window_size).map(|i| i as u16).collect(),  // Initialize with sequence
        }
    }
    // Drop is automatic for Vec
}
```

## Bridge Pattern for C/Rust Compatibility

For complex structs that need both C and Rust interfaces:

```rust
pub struct ZopfliHashBridge {
    #[cfg(feature = "c-fallback")]
    c_hash: Box<crate::ffi::ZopfliHashC>,
    #[cfg(not(feature = "c-fallback"))]
    rust_hash: crate::hash::ZopfliHash,
}

impl ZopfliHashBridge {
    pub fn new(window_size: usize) -> Self {
        // Route to C or Rust implementation based on feature flags
    }
}
```

# Porting, Building, Testing, and Debugging.

When porting a module, follow these steps:

## Initial Port

Following the guidelines above, write your new Rust module which duplicates the API of the C module.
Your Rust module should have a source file name which matches the C module name.
You may define common helper modules for e.g. FFI, error handling, etc as well.

## Rust Testing

Always build Rust in debug mode with sanitizers enabled. We will perform release
testing after the port is complete.

### Unit tests

Write inline unit tests in your Rust module as part of your initial port. These
should exercise all functions in the module. You may now move on to fuzz testing.

### Fuzz testing

(Fuzz testing may not be relevant for modules which do not take variable input.
Document this in your port document.)

Write a `{filename}_fuzz.rs` file which uses `rust-fuzz` to fuzz your Rust
module. Define the appropriate tests which use the `fuzz_target!` macro to
define the fuzz target. You may use the `arbitrary` crate to help generate test
data.

### C/FFI testing

You may now proceed to write a C test module which exercises your individual
Rust module. This should be named `test_rust_ffi_{filename}.c` and should be a
minimal C program which exercises the CFFI interface of your Rust module. This C
module should _explicitly_ link against your Rust code, and not use the default
build system.
