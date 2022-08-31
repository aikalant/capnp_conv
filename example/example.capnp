@0xd25a56591552e1e6;

struct BasicStruct {
    val @0 :Int32;
}

struct GenericStruct(T) {
    val @0 :T;
}

enum ExampleEnum {
    val1 @0;
    val2 @1;
}
struct ExampleStruct(T) {
  i32Val                @0   :Int32;
  textVal               @1   :Text;
  dataVal               @2   :Data;
  nestedVal             @3   :BasicStruct;
  enumVal               @4   :ExampleEnum;
  enumValRemote         @5   :ExampleEnum;
  genericStruct         @6   :GenericStruct(BasicStruct);
  genericGenericStruct  @7   :GenericStruct(T);
  listVal               @8   :List(List(GenericStruct(T)));
  groupVal :group {
    val1 @9  :T;
    val2 @10 :T;
  }
  unionVal :union {
    val1 @11 :T;
    val2 @12 :T;
  }
  union {
    val1 @13 :T;
    val2 @14 :T;
  }
}