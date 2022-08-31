fn main() {
    capnpc::CompilerCommand::new()
        .file("example.capnp")
        .run()
        .expect("compiling schema");
}
