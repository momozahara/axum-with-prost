fn main() {
    prost_build::compile_protos(&["html/book.proto"], &["html/"]).unwrap();
}
