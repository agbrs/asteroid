use agb_image_converter::{convert_image, Colour, ImageConverterConfig, TileSize};

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");
    convert_image(&ImageConverterConfig {
        transparent_colour: Some(Colour::from_rgb(26, 8, 14)),
        tile_size: TileSize::Tile8,
        input_image: "gfx/sprite_sheet.png".into(),
        output_file: format!("{}/sprite_sheet.rs", out_dir).into(),
    });
}
