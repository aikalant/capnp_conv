@0xbc97e80bcd46ea07;

struct BasicStruct {
  val @0 :Void;
}

struct GenericStruct(A,B) {
  aVal @0 :A;
  bVal @1 :B;
}

enum ComprehensiveStructEnum {
  val1 @0;
  val2 @1;
}

struct ComprehensiveStruct(T,Y) {
  struct NestedStruct {
    tVal @0 :T;
    yVal @1 :Y;
  }

  voidVal @0 :Void;
  boolVal @1 :Bool;
  i8Val @2 :Int8;
  i16Val @3 :Int16;
  i32Val @4 :Int32;
  i64Val @5 :Int64;
  u8Val @6 :UInt8;
  u16Val @7 :UInt16;
  u32Val @8 :UInt32;
  u64Val @9 :UInt64;
  f32Val @10 :Float32;
  f64Val @11 :Float64;
  textVal @12 :Text;
  dataVal @13 :Data;
  u8ListVal @14 :List(UInt8);
  nestedVal @15 :BasicStruct;
  listVal @16 :List(List(BasicStruct));
  enumVal @17 :ComprehensiveStructEnum;
  enumValRemote @18 :ComprehensiveStructEnum;
  groupVal :group {
    tVal @19 :T;
    yVal @20 :Y;
  }
  unionVal :union {
    tVal @21 :T;
    yVal @22 :Y;
  }
  union {
    tVal2 @23 :T;
    yVal2 @24 :Void;
  }
  tVal @25 :T;
  yVal @26 :Y;
  comprehensiveUnion :union {
    voidVal @27 :Void;
    boolVal @28 :Bool;
    i8Val   @29 :Int8;
    textVal @30 :Text;
    dataVal @31 :Data;
    tVal @32 :T;
    yVal @33 :Y;
    listVal @34 :List(List(BasicStruct));
    enumVal @35 :ComprehensiveStructEnum;
    enumValRemote @36 :ComprehensiveStructEnum;
    nestedVal @37 :BasicStruct;
    groupVal :group {
      tVal @38 :T;
      yVal @39 :Y;
    }
    unionVal :union {
      tVal @40 :T;
      yVal @41 :Y;
      genericVal @42 :GenericStruct(BasicStruct,BasicStruct);
    }
    genericVal @43 :GenericStruct(BasicStruct,BasicStruct);
  }
  genericVal @44 :GenericStruct(BasicStruct,BasicStruct);
}