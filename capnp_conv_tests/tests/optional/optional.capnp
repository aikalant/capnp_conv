@0xef09ef749da18e77;

struct TestOptional {
  prim @0 :Int32;
  struc @1 :BasicStruct;
  text @2 :Text;
  data @3 :Data;
  list @4 :List(Int32);
}

struct TestDefaults {
  prim @0 :Int32 = 999;
  struc @1 :BasicStruct = (val = 5);
  text @2 :Text = "default";
  data @3 :Data = 0x"00 01 02";
  list @4 :List(Int32) = [10, 9, 8];
}

struct BasicStruct {
  val @0 :Int32;
}