#[cfg(any(
    all(feature = "er", feature = "ds3"),
    all(feature = "ds3", feature = "ac6"),
    all(feature = "ac6", feature = "er")
))]
compile_error!("Only one of the target game features (ds3, er, ac6) may be enabled");

use std::{error::Error, time::Instant};

use field_metadata::{serialize_fb_repo, Block, FieldBlock, FieldBlockRepo};
use paramdex::{git_fetch::ParamdexGitFetch, Paramdex};

#[cfg(feature = "ds3")]
const GAME: &'static str = "DS3";
#[cfg(feature = "er")]
const GAME: &'static str = "ER";
#[cfg(feature = "ac6")]
const GAME: &'static str = "AC6";

const BLOCK_SIZE: usize = std::mem::size_of::<Block>();
const BLOCK_SIZE_BITS: usize = 8 * BLOCK_SIZE;

fn main() -> Result<(), Box<dyn Error>> {
    let log_conf = simple_log::LogConfigBuilder::builder()
        .output_file()
        .level(simple_log::log_level::DEBUG)
        .path("build_script.log")
        .build();
    simple_log::new(log_conf)?;

    log::info!("Starting ppatch build script...");

    let now = Instant::now();
    let paramdex_path = ParamdexGitFetch::new("https://github.com/vawser/Smithbox.git")
        .branch("1.0.18.1")
        .paramdex_path("src/StudioCore/Assets/Paramdex")
        .games(["DS3", "ER", "AC6"])
        .fetch_cached(".paramdex")?;

    log::info!(
        "Paramdex at {} fetched in {:?}",
        paramdex_path.to_string_lossy(),
        now.elapsed()
    );
    let now = Instant::now();

    let mut paramdex = Paramdex::new(paramdex_path.join(GAME));
    paramdex.load_defs()?.compute_def_layouts(u64::MAX);

    log::info!("{GAME} paramdefs loaded in {:?}", now.elapsed());
    let now = Instant::now();

    let mut fb_repo = FieldBlockRepo::new();
    for def in paramdex.defs() {
        assert!(def.fields.len() < u16::MAX as usize);
        let mut blocks: Vec<FieldBlock<Block>> = Vec::new();

        for f in def.fields.iter().filter(|f| f.bit_offset.is_some()) {
            let bofs = f.bit_offset.unwrap();
            let mut offset = (bofs / BLOCK_SIZE_BITS) as u16;

            let mask = Block::MAX >> (BLOCK_SIZE_BITS.saturating_sub(f.size_bits()))
                << (bofs - (bofs & BLOCK_SIZE_BITS - 1));

            let field_start = blocks.len() as u16;
            blocks.push(FieldBlock {
                field_start,
                offset,
                mask,
            });

            let mut remaining_bits = f.size_bits() - mask.count_ones() as usize;
            while remaining_bits != 0 {
                let mask = Block::MAX >> (BLOCK_SIZE_BITS.saturating_sub(remaining_bits));
                blocks.push(FieldBlock {
                    field_start,
                    offset,
                    mask,
                });
                offset += 1;
                remaining_bits -= mask.count_ones() as usize;
            }
        }

        assert!(blocks.len() < u16::MAX as usize);
        fb_repo.insert(def.param_type.clone(), blocks);
    }

    let serialized = serialize_fb_repo(&fb_repo);
    std::fs::write("field_blocks.bin", &serialized)?;
    log::info!("{GAME} field blocks built in {:?}", now.elapsed());

    println!("cargo:rerun-if-changed=.paramdex");
    println!("cargo:rerun-if-changed=../paramdex");

    Ok(())
}
