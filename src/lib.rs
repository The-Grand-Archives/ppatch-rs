#[cfg(any(
    all(feature = "er", feature = "ds3"),
    all(feature = "ds3", feature = "ac6"),
    all(feature = "ac6", feature = "er")
))]
compile_error!("Only one of the target game features (ds3, er, ac6) may be enabled");

mod celua;
mod from;
mod param_file;
mod patchers;
mod util;
mod vtable;
