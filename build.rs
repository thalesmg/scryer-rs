fn main() {
    let language = "erlang";
    let package = format!("tree-sitter-{}", language);
    let source_directory = format!("vendor/{}/src", package);
    let source_file = format!("{}/parser.c", source_directory);

    println!("cargo:rerun-if-changed={}", source_file);

    cc::Build::new()
        .file(source_file)
        .include(source_directory)
        .compile(&package);
}
