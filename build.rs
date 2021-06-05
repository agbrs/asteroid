fn main() {
    let out_file_name = "graphics";
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");
    let out_file_path = format!("{}/{}", out_dir, &out_file_name);

    let out = std::process::Command::new("grit")
        .args(&["gfx/ship.png"])
        .args(&[
            "-p",
            "-pS",
            &format!("-O{}", out_file_path),
            "-fh!",
            "-ftb",
            "-gB4",
        ])
        .output()
        .expect("failed to make images");

    if !out.status.success() {
        panic!("{}", String::from_utf8_lossy(&out.stderr));
    }

    let out = std::process::Command::new("bash")
        .arg("-c")
        .arg(format!(
            "cat ship.img.bin > {out}/graphics.img.bin",
            out = out_dir
        ))
        .output()
        .expect("failed to make images");
    if !out.status.success() {
        panic!("{}", String::from_utf8_lossy(&out.stderr));
    }
    let out = std::process::Command::new("bash")
        .arg("-c")
        .arg("rm ship.img.bin")
        .output()
        .expect("failed to make images");
    if !out.status.success() {
        panic!("{}", String::from_utf8_lossy(&out.stderr));
    }
}
