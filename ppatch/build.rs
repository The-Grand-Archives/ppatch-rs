#[cfg(any(
    all(feature = "er", feature = "ds3"),
    all(feature = "ds3", feature = "ac6"),
    all(feature = "ac6", feature = "er")
))]
compile_error!("Only one of the target game features (ds3, er, ac6) may be enabled");

use std::time::Instant;

use build_utils::paramdex_fetch::ParamdexGitFetch;

fn main() {
    println!("cargo:rerun-if-changed=.paramdex");
    println!("cargo:rerun-if-changed=../build_utils");
    println!("cargo:rerun-if-changed=../codegen");

    let now = Instant::now();
    let paramdex_path = ParamdexGitFetch::new("https://github.com/vawser/Smithbox.git")
        .branch("1.0.18.1")
        .paramdex_path("src/StudioCore/Assets/Paramdex")
        .games(["DS3", "ER", "AC6"])
        .fetch_cached(".paramdex")
        .unwrap();

    println!(
        "cargo:warning=Paramdex: {} (fetched in {:?})",
        paramdex_path.to_string_lossy(),
        now.elapsed()
    );
}
