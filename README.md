# capnp_conv

The `capnp_conv` macro facilitates conversions between [Cap'n Proto](https://capnproto.org/) structs/unions/unions/groups to their rust counterparts by implementing all the necessary [Capn' Proto Rust](https://github.com/capnproto/capnproto-rust) generated builder/reader calls.

Inspired by the (seemingly abandoned?) [existing PR](https://github.com/capnproto/capnproto-rust/pull/157) by @realcr.

## Usage
The following capnp schema file directly translates to the below rust file:

```capnp
struct SomeStruct { }

enum CapnpEnum {
    val1 @0;
    val2 @1;
}

struct CapnpStruct {
  voidVal   @0   :Void;
  i32Val    @1   :Int32;
  textVal   @2   :Text;
  dataVal   @3   :Data;
  structVal @4   :SomeStruct;
  enumVal   @5   :CapnpEnum;
  listVal   @6   :List(SomeStruct);
}
```
```rust
#[capnp_conv(some_struct)]
pub struct SomeStruct {}

#[capnp_conv(capnp_struct)]
pub struct RustStruct {
  void_val: (),
  i32_val: i32,
  text_val: String,
  #[capnp_conv(type = "data")]
  data_val: Vec<u8>,
  struct_val: SomeStruct,
  #[capnp_conv(type = "enum")]
  enum_val: CapnpEnum,
  list_val: Vec<SomeStruct>,
}
```

The `capnp_conv` proc macro implements the `capnp_conv::Readable`, `capnp_conv::Writable`, and `TryFrom<Reader>` traits, which handle all of the reading/writing:

```rust
fn read(reader: capnp_struct::Reader) -> Result<RustStruct, capnp::Error> {
  RustStruct::read(reader)?
}
fn write(rust_struct: RustStruct, builder: capnp_struct::Builder) {
  rust_struct.write(builder)
}
```

## Special Type Handling

Capnp `group`, `enum`, `union`, and `data` types require the field attribute with a type specifier.

### Groups
Capnp `group`s are represented by separate rust `struct`s.

```capnp
struct CapnpStruct {
    groupVal :group {
        val @0 :Void;
    }
}
```
```rust
#[capnp_conv(capnp_struct)]
pub struct RustStruct {
  #[capnp_conv(type = "group")]
  group_val: RustStructGroup,
}
#[capnp_conv(capnp_struct::group_val)]
pub struct RustStructGroup {
  val: ()
}
```

### Enums
The macro has two options when it comes to enums: `enum` and `enum_remote`. Because code generated capnp files already contain enum definitions, it is possible to use them directly with `enum`. This eliminates the need to write an extra enum definition, but there are some times when it is useful to define a separate enum, for example, if it is necessary to derive traits or use other macros on the enum. For these cases, use `enum_remote` with a separately defined rust enum. When used on a rust enum, the `capnp_conv` macro generates the `from`/`into` trait implementations for its capnp counterpart.

```capnp
enum CapnpEnum {
    val1 @0;
    val2 @1;
}
struct CapnpStruct {
    enumVal       @0 :CapnpEnum;
    enumValRemote @1 :CapnpEnum;
}
```
```rust
#[capnp_conv(CapnpEnum)]
pub enum RustEnum {
  Val1,
  Val2,
}
#[capnp_conv(capnp_struct)]
pub struct RustStruct {
  #[capnp_conv(type = "enum")]
  enum_val: CapnpEnum,
  #[capnp_conv(type = "enum_remote")]
  enum_val_remote: RustEnum,
}
```

### Unions
Unions can be represented in two different ways. One is by using rust `enum`s, but `struct`s can also represent capnp unions with fields wrapped in `Option<T>`s and containing the `#[capnp_conv(union_variant)]` attribute macro. This eliminates the need to create a separate item for unnamed unions, but it can be more cumbersome to manipulate.

Rust enum variants have the same requirements for using the `#[capnp_conv(type = xxx)]` field attributes.

```capnp
struct CapnpStruct {
    namedUnion :union {
        val1 @0 :Void;
        val2 @1 :Void;
    }
    union {
        val1 @1 :Void;
        val2 @2 :Void;
    }
}
```
```rust
#[capnp_conv(capnp_struct)]
pub struct RustStruct {
  #[capnp_conv(type = "union")]
  named_union: RustStructUnion,
  #[capnp_conv(type = "unnamed_union")]
  unnamed_union: RustStructUnamedUnion,
}
#[capnp_conv(capnp_struct::named_union)]
pub enum RustStructUnion {
  Val1(()),
  Val2(()).
}
#[capnp_conv(capnp_struct)]
pub enum RustStructUnamedUnion {
  Val1(()),
  Val2(()).
}

// or

#[capnp_conv(capnp_struct)]
pub struct RustStruct {
  #[capnp_conv(type = "union")]
  named_union: RustStructUnion,
  #[capnp_conv(union_variant)]
  val1: Option<()>,
  #[capnp_conv(union_variant)]
  val2: Option<()>,
}
#[capnp_conv(capnp_struct::named_union)]
pub struct RustStructUnion {
  #[capnp_conv(union_variant)]
  val1: Option<()>,
  #[capnp_conv(union_variant)]
  val2: Option<()>,
}
```

### Data
The capnp `Data` type is functionally identical to `List(UInt8)`, both of which are represented with `Vec<u8>` in rust. However, capnpc generates two distinct structs that handle reading and writing of the two types. There is no way to specify which one a `Vec<u8>` is intended to represent, which necessitates the use of the field attribute in the case of `Data` types.

```capnp
struct CapnpStruct {
    named_union :union {
        list @0 :List(UInt8);
        data @1 :Data;
    }
}
```
```rust
#[capnp_conv(capnp_struct)]
pub struct RustStruct {
  list: Vec<u8>,
  #[capnp_conv(type = "data")]
  data: Vec<u8>,
}
```

## Extra Features

`capnp_conv` includes several other features that can be enabled through setting options in field attributes

### Renaming fields
Rust field names must normally match the field names of their capnp counterparts (converted to the appropriate rust case). Using the `name` attribute, it is possible to disentangle them.

```capnp
struct CapnpStruct {
    capVal @0 :Void;
    capnpUnion :union {
        capVal1 @1 : Void;
        capVal2 @2 : Void;
    }
}
```
```rust
#[capnp_conv(capnp_struct)]
pub struct RustStruct {
  #[capnp_conv(name = "capVal")] //or #[capnp_conv(name = "cap_val")]
  arbitrary_name: (),
  #[capnp_conv(name = "capnp_union")]
  #[capnp_conv(type = "union")]
  rust_union: RustUnnion,
}
#[capnp_conv(capnp_struct::capnp_union)]
pub enum RustUnion {
  #[capnp_conv(name = "capVal1")]
  ArbitraryName1(())
  #[capnp_conv(name = "cap_val2")]
  ArbitraryName2(())
}
```

### Optional fields
Wrapping a field with `Option<T>` indicates that the field is optional.
When a struct is written to a builder, optional fields will be skipped if they are `None`.
When a struct is read from a reader, optional fields that are pointer types (`List`, `Struct`, `Text`, `Data`) will be set to `None` if the field has not been set in the capnp message. Primitive types and enums will always be read and set to `Some`.

- Unions cannot contain optional fields.
- Groups and unions cannot be optional.
```capnp
struct CapnpStruct {
    capVal @0 :Void;
}
```
```rust
#[capnp_conv(capnp_struct)]
pub struct RustStruct {
  cap_val: Option<()>,
}
```

### Skipped fields
- Adding `#[capnp_conv(skip_write)]` to a field's attributes will result in the field not being written. Similar to as if it were optional with `None`.

- Adding `#[capnp_conv(skip_read)]` to a field's attributes will result in the field never being read. During a read, it be set to the field type's default value (note the field type must implement the `Default` trait), or `None` if the field is optional. This can be futher configured by setting the field's `default` attribute (see below).

- `#[capnp_conv(skip)]` skips both reading and writing.

- Unions cannot contain skipped fields.

### Default override fields
 
Setting the `#[capnp_conv(default = "path_to_func_to_call")]` attribute to a field that is configured as `skip_read` or `skip` will set the field to the output of calling the function specified rather than the default value during a read.

Unions cannot contain default overrides.

### Generics

Generics are supported for both structs and enums.

```capnp
struct CapnpStruct(T,Y) {
  tVal @0 :T;
  list @1 :List(Y);
  unionVal :union {
      tVal @2 :T;
      list @3 :List(Y);
  }
}
```
```rust
#[capnp_conv(capnp_struct)]
pub struct RustStruct<T,Y> {
  t_val: T,
  list: Vec<Y>,
  #[capnp_conv(type = "union")]
  union_val: RustStructUnionVal<T,Y>,
}
#[capnp_conv(capnp_struct::union_val)]
pub enum RustStructUnionVal<T,Y> {
  TVal(T),
  List(Vec<Y>),
}
```

Limitations:

One feature of capnp schemas that is not easily reproduced in rust is nested struct definitions. This is not typically an issue as they can be implemented as flattened rust structs, but when combined with the fact that nested capnp structs/unions/groups have access to all the generic types of all of their ancestors, this can be problematic for rust models.

For example:
```capnp
struct ParentStruct(T,Y,R) {
    tVal @0 :T;
    yVal @1 :Y;
    rVal @2 :R;
    unionVal :union {
        voidVal @3 :Void,
        tVal    @4 :T,
    }

    struct ChildStruct {
        tVal @0 :T
    }
}
```
 When defining `parent_struct::child_struct` in rust, the struct must have `T`, `Y`, and `R` even though it only uses `T`. Rust does not allow structs to have generics that they do not use, but we can use [PhantomData<T>](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) to overcome this:
 ```rust
 pub struct ChildStruct<T,Y,R> {
  t_val: T,
  phantom: PhantomData<*const (Y,R)>,
 }
 ```

 When reading/writing, `PhantomData` types will be automatically skipped, even without the field attribute.

 Unfortunately there is no "PhantomVariant" for rust enums, so for unions, instead make the second field of any variant a `PhantomData`:

 ```rust
 pub enum UnionVal(T,Y,R) {
   VoidVal((), PhantomData<*const (Y,R)>),
   TVal(T),
 }
 ``` 

 ## Future work

Short term:
- Add support for top level `Vec` read and write
- Add support for `anypointer` type
- Add more validations to allow the compiler to provide more clues when the macro is not properly used.
   - unions must have at least 2 fields (both enums and struct unions apply)
   - cannot have more than 1 unnamed union
   - lists cannot have generic types as their type
   - enums must either all be unit types or complex. no mix & match
   - assert fields with type specifiers are not primitives, blobs, void, or lists (could be done in regular parsing)
   - fields with `type = "data"` attribute must be of type `Vec<u8>`
- Finish writing tests. Priority needs:
   - skipped and default fields
   - union struct representation
- Confirm if as_turbofish() function is sufficient for all possible cases (specifically, nested generic types? `Type1<Type2<T>>`)

Long term:
- Allow boxed fields for the rare case of a struct/enum containing a field of its own type (with different generics).
- Allow structs to contain lifetimes and generic constraints that carry over to the generated impls.
- Add a convenience `clear_enum_fields` function to struct represented capnp unions that sets all union fields to `None`.