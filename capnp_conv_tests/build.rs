fn main() {
    capnpc::CompilerCommand::new()
        .file("tests/test.capnp")
        .src_prefix("tests/")
        .run()
        .expect("compiling schema");
}
