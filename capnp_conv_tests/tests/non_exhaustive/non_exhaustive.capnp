@0xc98c4483cf345960;

struct TestUnion {
  prim @0 :Int32;
  union {
    unionVal1 @1 :Void;
    unionVal2 @2 :Void;
    extra @3 :Void;
  }
}

struct TestUnionPure {
  union {
    unionVal1 @0 :Void;
    unionVal2 @1 :Void;
    extra @2 :Void;
  }
}

enum TestEnum {
  val1 @0;
  val2 @1;
  extra @2;
}

struct TestStruct {
  testEnum @0: TestEnum;
  testEnumRemote @1: TestEnum;
  testUnion @2: TestUnion;
  testUnionPure @3: TestUnionPure;
}