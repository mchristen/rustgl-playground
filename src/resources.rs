use std::fs;
use std::io;
use std::io::Read;
use std::ffi;

use std::path::{Path, PathBuf};

use failure::Fail;
use font_kit::font::Font;
use font_kit::file_type::FileType;
use font_kit::handle::Handle;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "I/O Error")]
    Io(#[cause] io::Error),
    #[fail(display = "File is malformed(containes a null byte)")]
    MalformedFile,
    #[fail(display = "Path lookup failed")]
    NoExePath,
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}

pub struct Resources {
    root: PathBuf,
}

impl Resources {
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn from_relative_exe(path: &Path) -> Result<Resources, Error> {
        let exe_file = ::std::env::current_exe().map_err(|_| Error::NoExePath)?;
        let exe_path = exe_file.parent().ok_or(Error::NoExePath)?;
        Ok(Resources {
            root: exe_path.join(path)
        })
    }

    pub fn load_font(&self, resource_name: &str) -> Result<Vec<Font>, Error> {
        let mut font_handles = vec!();
        let normalized_path = normalize_resource_path(&self.root, resource_name);
        println!("{:?}", normalized_path);

        let mut file = fs::File::open(normalized_path.to_owned())?;
        match Font::analyze_file(&mut file) {
            Err(_) => panic!("Error loading font file from path: {:?}", normalized_path.to_owned()),
            Ok(FileType::Single) => font_handles.push(Handle::from_path(normalized_path.to_owned(), 0)),
            Ok(FileType::Collection(font_count)) => {
                for font_index in 0..font_count {
                    font_handles.push(Handle::from_path(normalized_path.to_owned(), font_index))
                }
            }
        }
        let mut fonts = vec!();
        for handle in &font_handles {
            fonts.push(Font::from_handle(&handle).unwrap());
        }
        Ok(fonts)
    }
    
    pub fn load_cstring(&self, resource_name: &str) -> Result<ffi::CString, Error> {
        let normalized_path = normalize_resource_path(&self.root, resource_name);
        println!("{:?}", normalized_path);
        let mut file = fs::File::open(normalized_path)?;
        let mut buffer: Vec<u8> = Vec::with_capacity(
            file.metadata()?.len() as usize + 1
        );
        file.read_to_end(&mut buffer)?;

        if buffer.iter().find(|i| **i == 0).is_some() {
            return Err(Error::MalformedFile);
        }

        Ok(unsafe { ffi::CString::from_vec_unchecked(buffer) })
    }
}

fn normalize_resource_path(root: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root.into();

    for part in location.split("/") {
        path = path.join(part);
    }

    path
}