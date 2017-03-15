#![feature(field_init_shorthand)]
#![feature(try_from)]
#![feature(custom_attribute)]
#![feature(slice_patterns)]
#![feature(associated_consts)]
#![feature(associated_type_defaults)]
#![feature(conservative_impl_trait)]
#![recursion_limit = "1024"]

extern crate rayon;
extern crate crypto;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub type Error = Box<::std::error::Error + Send + Sync>;
pub type Result<T> = ::std::result::Result<T, Error>;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn checksum<T: AsRef<Path>>(path: T) -> Result<String> {
    use crypto::digest::Digest;
    use std::io::Read;
    const CAP: usize = 1024 * 128;
    let mut buffer = [0u8; CAP];
    let mut file = File::open(path)?;
    let mut md5 = ::crypto::md5::Md5::new();
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 { break; }
        md5.input(&buffer[..n]);
    }
    Ok(md5.result_str())
}

#[derive(Deserialize, Debug)]
pub struct FfmpegInfoFormat {
    duration: String
}

#[derive(Deserialize, Debug)]
pub struct FfmpegInfoStream {
    pub codec_type: String,
    pub width: Option<u64>,
    pub height: Option<u64>
}

#[derive(Deserialize, Debug)]
pub struct FfmpegInfo {
    pub streams: Vec<FfmpegInfoStream>,
    pub format: FfmpegInfoFormat
}

pub fn info<T: AsRef<Path>>(path: T) -> Result<FfmpegInfo> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(&*path.as_ref().to_string_lossy())
        .output()?;

    Ok(::serde_json::from_slice(&output.stdout)?)
}

pub fn thumbnail<T: AsRef<Path>>(
    path: T,
    dest: T,
    at: Option<usize>,
    width: Option<i64>,
    height: Option<i64>) -> Result<String> {

    let path = path.as_ref();

    let name = {
        let stem = path.file_stem().ok_or("Couldn't determine file stem")?;
        let qual = [
            width.map(|x| format!("{}", x) ).unwrap_or("".into()),
            height.map(|x| format!("{}", x) ).unwrap_or("".into())].join("x");
        
        format!("{}_{}.jpg", stem.to_string_lossy(), qual)
    };

    let output = dest.as_ref().join(&name);

    let status = Command::new("ffmpeg")            
        .arg("-ss")
        .arg(&format!("{}", at.unwrap_or(5)))
        .arg("-i")
        .arg(path)
        .arg("-vframes")
        .arg("1")
        .arg("-q:v")
        .arg("2")
        .arg("-v")
        .arg("quiet")
        .arg("-vf")
        .arg(&format!("scale={}:{}", width.unwrap_or(-1), height.unwrap_or(-1)))
        .arg(&output)
        .status()?;

    if status.success() { 
        Ok(name) 
    } else { 
        Err("Couldn't execute command".into())
    }
}

fn main() {
    
    let new_path = ::std::env::var("PATHX").unwrap();
    let folder = ::std::env::var("DEST").unwrap();

    let ((info, checksum), (thumb_sm, thumb_lg)) = ::rayon::join(
        || ::rayon::join(
            || info(&new_path),
            || checksum(&new_path)
        ),
        || ::rayon::join(
            || thumbnail(&new_path, &folder, Some(5), Some(192), None),
            || thumbnail(&new_path, &folder, Some(5), Some(420), None)
        )
    );

    let (info, checksum, thumb_sm, thumb_lg) = 
        (info.unwrap(), checksum.unwrap(), thumb_sm.ok(), thumb_lg.ok());

    

}
