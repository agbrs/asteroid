use agb_image_converter::{convert_image, Colour, ImageConverterConfig, TileSize};

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");
    convert_image(
        ImageConverterConfig::builder()
            .tile_size(TileSize::Tile8)
            .transparent_colour(Colour::from_rgb(26, 8, 14))
            .input_image("gfx/sprite_sheet.png".into())
            .output_file(format!("{}/sprite_sheet.rs", out_dir).into())
            .build(),
    );
    convert_image(
        ImageConverterConfig::builder()
            .tile_size(TileSize::Tile8)
            .transparent_colour(Colour::from_rgb(26, 8, 14))
            .input_image("gfx/tilemap.png".into())
            .output_file(format!("{}/background_sheet.rs", out_dir).into())
            .build(),
    );
}
