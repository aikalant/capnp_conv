@0xd922edaf981d3d69;

struct UnionStruct {
  prim @0 :Int32;
  union {
    unionVal1 @1 :Text;
    unionVal2 @2 :BasicStruct;
  }
}

struct UnionStructPure {
  union {
    unionVal1 @0 :Text;
    unionVal2 @1 :BasicStruct;
  }
}

struct BasicStruct {
  val @0 :Int32;
}