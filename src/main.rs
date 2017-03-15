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

pub fn thumbnail<T: AsRef<Path>>(path: T, i: usize, dest: T) -> Result<String> {

    let path = path.as_ref();

    let name = {
        let stem = path.file_stem().ok_or("Couldn't determine file stem")?;
        format!("{}_{}_thumb.jpg", stem.to_string_lossy(), i)
    };

    let output = dest.as_ref().join(&name);

    let status = Command::new("ffmpeg")            
        .arg("-ss")
        .arg("5")
        .arg("-i")
        .arg(path)
        .arg("-vframes")
        .arg("1")
        .arg("-q:v")
        .arg("2")
        .arg("-v")
        .arg("quiet")
        .arg("-vf")
        .arg("scale=192:-1")
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
            || thumbnail(&new_path, 1, &folder),
            || thumbnail(&new_path, 2, &folder)
        ),
        || ::rayon::join(
            || thumbnail(&new_path, 3, &folder),
            || thumbnail(&new_path, 4, &folder)
        )
    );

    let (info, checksum, thumb_sm, thumb_lg) = 
        (info.unwrap(), checksum.unwrap(), thumb_sm.ok(), thumb_lg.ok());

    

}
