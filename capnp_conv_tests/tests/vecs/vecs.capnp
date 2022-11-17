@0xe797bc6b7bf75fcb;

struct WrapperStruct {
  primitiveList @0 :List(Int32);
  structList @1 :List(BasicStruct);
  textList @2 :List(Text);
  dataList @3 :List(Data);
  enumList @4: List(BasicEnum);
  enumRemoteList @5: List(BasicEnum);
  #listList @6 :List(List(Int32));
}

struct BasicStruct {
  val @0 :Int32;
}

enum BasicEnum {
  val1 @0;
  val2 @1;
}

struct DataTestStruct {
  u8List @0 :List(UInt8);
  dataList @1 :List(Data);
}