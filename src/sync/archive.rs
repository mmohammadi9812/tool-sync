use flate2::read::GzDecoder;
use std::fs;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use tar;

pub struct Archive<'a> {
    archive_path: &'a PathBuf,
    tmp_dir: &'a Path,
    exe_name: &'a str,
    archive_type: ArchiveType<'a>,
}

/// Archive type that specifies how to unpack asset
enum ArchiveType<'a> {
    Zip(&'a str),
    TarGz(&'a str),
}

impl<'a> Archive<'a> {
    pub fn from(archive_path: &'a PathBuf, tmp_dir: &'a Path, exe_name: &'a str, asset_name: &'a str) -> Option<Archive<'a>> {
        let tar_gz_dir = asset_name.strip_suffix(".tar.gz");

        match tar_gz_dir {
            Some(tar_gz_dir) => Some(Archive {
                archive_path,
                tmp_dir,
                exe_name,
                archive_type: ArchiveType::TarGz(tar_gz_dir),
            }),
            None => {
                let zip_dir = asset_name.strip_suffix(".zip");

                match zip_dir {
                    Some(zip_dir) => Some(Archive {
                        archive_path,
                        tmp_dir,
                        exe_name,
                        archive_type: ArchiveType::Zip(zip_dir),
                    }),
                    None => None,
                }
            }
        }
    }

    /// Unpack archive and return path to the executable tool
    pub fn unpack(&self) -> Result<PathBuf, std::io::Error> {
        match self.archive_type {
            ArchiveType::TarGz(asset_name) => unpack_tar(
                self.archive_path,
                self.tmp_dir,
                self.exe_name,
                asset_name,
            ),
            ArchiveType::Zip(asset_name) => unpack_zip(
                self.archive_path,
                self.tmp_dir,
                self.exe_name,
                asset_name,
            ),
        }
    }

}

fn unpack_tar(tar_path: &PathBuf, tmp_dir: &Path, exe_name: &str, asset_name: &str) -> Result<PathBuf, std::io::Error> {
    println!("Tar path: {}", tar_path.display());
    println!("Tar path exists: {}", tar_path.is_file());

    // unpack tar_path to tmp_dir
    let tar_file = File::open(tar_path)?;
    let tar_decoder = GzDecoder::new(tar_file);
    let mut archive = tar::Archive::new(tar_decoder);
    archive.unpack(tmp_dir)?;

    // create path to the final executable
    let mut tool_path = PathBuf::new();
    tool_path.push(tmp_dir);
    tool_path.push(asset_name);
    tool_path.push(exe_name);

    Ok(tool_path)
}

fn unpack_zip(zip_path: &PathBuf, tmp_dir: &Path, exe_name: &str, asset_name: &str) -> Result<PathBuf, std::io::Error> {
    let zipfile = File::open(&zip_path)?;

    let mut archive = zip::ZipArchive::new(zipfile)?;

    let exe_path = format!("bin/{exe_name}");
    let mut input_file = archive.by_name(&exe_path)?;

    // create path to the final executable
    let mut tool_path = PathBuf::new();
    tool_path.push(tmp_dir);
    tool_path.push(exe_name);

    // Create file for the output path
    let mut output_file = fs::File::create(&tool_path)?;

    // write content to the output path
    io::copy(&mut input_file, &mut output_file)?;

    Ok(tool_path)
}
