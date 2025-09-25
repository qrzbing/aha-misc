extern crate cc;

fn main() {
    println!("cargo:rerun-if-changed=src/c/src/utils.c");
    cc::Build::new().file("src/c/utils.c").compile("utils.a");
}
