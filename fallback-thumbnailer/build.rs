use std::env;
use std::fs::{self, File};
use std::io::{Write};
use std::path::Path;
use zip::read::ZipArchive;
use reqwest::blocking::get;

fn main() {
    let archive_name = "ffmpeg-n7.0.2-13-g51482627ca-win64-gpl-shared-7.0";
    let zip_file_name = format!("{}.zip", archive_name);

    let out_dir_string = env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_dir = Path::new(&out_dir_string);

    let extract_path = out_dir.join(&archive_name);

    if extract_path.exists() {
        fs::remove_dir_all(&extract_path).expect("Failed to remove existing directory");
    }

    let url = format!("https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2024-09-02-12-48/{}", zip_file_name);
    let zip_path = out_dir.join(&zip_file_name);

    println!("Downloading from URL: {}", url);
    println!("Saving to file: {}", zip_path.display());

    let response = get(&url).expect("Failed to download the archive");
    let mut file = File::create(&zip_path).expect("Failed to create archive file");
    file.write_all(&response.bytes().expect("Failed to read response body")).expect("Failed to write to archive file");

    println!("Extracting archive to: {}", extract_path.display());

    let file = File::open(&zip_path).expect("Failed to open ZIP file");
    let mut archive = ZipArchive::new(file).expect("Failed to read ZIP archive");

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("Failed to access file in ZIP archive");
        let out_path = extract_path.join(file.name());

        if (&*file.name()).ends_with('/') {
            fs::create_dir_all(&out_path).expect("Failed to create directory");
        } else {
            if let Some(p) = out_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).expect("Failed to create directory");
                }
            }
            let mut outfile = File::create(&out_path).expect("Failed to create file");
            std::io::copy(&mut file, &mut outfile).expect("Failed to copy contents");
        }
    }

    let bin_dir = extract_path.join(archive_name).join("bin");
    println!("Setting bin_dir: {}", bin_dir.display());

    if bin_dir.exists() {
        unsafe {
            env::set_var("FFMPEG_DIR", bin_dir.to_str().expect("Failed to convert path to string"));
        }
        println!("FFMPEG_DIR is set to: {}", bin_dir.display());
    } else {
        panic!("No 'bin' directory found in the extracted contents");
    }

    fs::remove_file(&zip_path).expect("Failed to remove zip file");
    eprintln!("Removed archive file: {}", zip_path.display());

    eprintln!("cargo:rustc-env=FFMPEG_DIR={}", env::var("FFMPEG_DIR").expect("Failed to get FFMPEG_DIR"));

    for entry in fs::read_dir(bin_dir).expect("Failed to read bin_dir") {
        let entry = entry.expect("Failed to get entry");
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "dll" {
                    let dest_path = out_dir.parent().unwrap().parent().unwrap().parent().unwrap().join(entry.file_name());
                    println!("Copying {} to {}", path.display(), dest_path.display());
                    fs::copy(&path, &dest_path).expect("Failed to copy DLL");
                }
            }
        }
    }
}
