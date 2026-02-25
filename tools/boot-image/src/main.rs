use std::path::PathBuf;

fn main() {
    let kernel_path = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: boot-image <kernel-elf-path>"),
    );

    let out_dir = kernel_path.parent().unwrap();
    let bios_path = out_dir.join("minios-bios.img");

    let bios_image = bootloader::BiosBoot::new(&kernel_path);
    bios_image
        .create_disk_image(&bios_path)
        .expect("failed to create BIOS disk image");

    println!("BIOS image: {}", bios_path.display());
}
