//! a simple database that serializes to/from messagepack
use log::*;
use std::{
    io::prelude::*,
    io::SeekFrom,
    fs::{self, File, OpenOptions},
    marker::PhantomData,
    path::PathBuf,
    default::Default,
};
use flate2::{
    Compression,
    write::DeflateEncoder,
    read::DeflateDecoder,
};
use serde::{
    Serialize,
    de::DeserializeOwned
};
use failure::Fail;

#[derive(Debug)]
crate struct SimpleDB<D: DeserializeOwned + Serialize + Default> {
    path: PathBuf,
    _marker: PhantomData<D>,
}
// TODO: Figure out a way to use MessagePack instead of JSON
// JSON is OK because we compress it
// compression bench: of ETH tipjar addr txs, block 0-6mil - uncompressed 100MB, compressed 3.9MB
/// A simple DB that allows saving/retrieving structures to/from a (compressed) file,
impl<D> SimpleDB<D> where D: DeserializeOwned + Serialize + Default {
    crate fn new(path: PathBuf) -> Result<Self, DBError> {
        if !path.as_path().exists() {
            File::create(path.as_path())?;
        }
        Ok(SimpleDB {
            path,
            _marker: PhantomData
        })
    }

    /// Save structure to a file, serializing to JSON and then compressing with DEFLATE
    crate fn save(&self, data: D) -> Result<(), DBError> {
        self.mutate(|file| {
            let ser_data = serde_json::ser::to_vec(&data)?;
            let mut e = DeflateEncoder::new(file, Compression::default());
            e.write_all(ser_data.as_slice())?;
            e.finish()?;
            Ok(())
        })?;
        Ok(())
    }

    /// Get structure from file, DEFLATING and then deserializing from JSON
    crate fn get(&self) -> Result<D, DBError> {
        let meta = fs::metadata(self.path.as_path())?;
        if meta.len() == 0 {
            info!("File length is 0");
            return Ok(D::default());
        }
        self.read(|file| {
            let mut deflater = DeflateDecoder::new(file);
            let mut s = String::new();
            let bytes_read = deflater.read_to_string(&mut s)?;
            info!("Read {} bytes from database file", bytes_read);
            Ok(serde_json::de::from_str(&s)?)
        })
    }

    // open backend
    fn open(&self) -> Result<File, DBError> {
        Ok(OpenOptions::new().create(true).read(true).write(true).open(self.path.as_path())?)
    }

    // mutate the file, always setting seek back to beginning
    fn mutate<F>(&self, mut fun: F) -> Result<(), DBError>
    where
        F: FnMut(&mut File) -> Result<(), DBError>
    {
        let mut file = self.open()?;
        fun(&mut file)?;
        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    fn read<F>(&self, fun: F) -> Result<D, DBError>
    where F: Fn(&File) -> Result<D, DBError>
    {
        let mut file = self.open()?;
        let ret = fun(&file)?;
        file.seek(SeekFrom::Start(0))?;
        Ok(ret)
    }
}


#[derive(Fail, Debug)]
pub enum DBError {
    #[fail(display = "Could Not Find {}", _0)]
    NotFound(String),
    #[fail(display = "IO Error {}", _0)]
    IO(#[fail(cause)] std::io::Error),
    #[fail(display = "Could not decode: {}", _0)]
    Decode(#[fail(cause)] rmp_serde::decode::Error),
    #[fail(display = "Could not encode: {}", _0)]
    Encode(#[fail(cause)] rmp_serde::encode::Error),
    #[fail(display = "Bincode Decode/Encode failed {}", _0)]
    Bincode(#[fail(cause)] Box<bincode::Error>),
    #[fail(display = "Could not decode from JSON {}", _0)]
    SerdeJsonDecode(#[fail(cause)] serde_json::error::Error),
    #[fail(display = "Could not decode RON {}", _0)]
    RonDecode(#[fail(cause)] ron::de::Error),
    #[fail(display = "Could not encode RON {}", _0)]
    RonEncode(#[fail(cause)] ron::ser::Error)
}
impl From<ron::de::Error> for DBError {
    fn from(err: ron::de::Error) -> DBError {
        DBError::RonDecode(err)
    }
}

impl From<ron::ser::Error> for DBError {
    fn from(err: ron::ser::Error) -> DBError {
        DBError::RonEncode(err)
    }
}

impl From<Box<bincode::Error>> for DBError {
    fn from(err: Box<bincode::Error>) -> DBError {
        DBError::Bincode(err)
    }
}

impl From<serde_json::error::Error> for DBError {
    fn from(err: serde_json::error::Error) -> DBError {
        DBError::SerdeJsonDecode(err)
    }
}

impl From<std::io::Error> for DBError {
    fn from(err: std::io::Error) -> DBError {
        DBError::IO(err)
    }
}

impl From<rmp_serde::decode::Error> for DBError {
    fn from(err: rmp_serde::decode::Error) -> DBError {
        DBError::Decode(err)
    }
}

impl From<rmp_serde::encode::Error> for DBError {
    fn from(err: rmp_serde::encode::Error) -> DBError {
        DBError::Encode(err)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn save() {
        pretty_env_logger::try_init();
        let db = SimpleDB::<HashMap<String, usize>>::new(PathBuf::from("/tmp/SOME")).unwrap();
        let mut data = HashMap::new();
        data.insert("Hello".to_string(), 45);
        data.insert("Byte".to_string(), 34);
        db.save(data.clone()).unwrap();
    }

    #[test]
    fn get() {
        pretty_env_logger::try_init();
        let db = SimpleDB::<HashMap<String, usize>>::new(PathBuf::from("/tmp/SOME")).unwrap();
        let mut data = HashMap::new();
        data.insert("Hello".to_string(), 45);
        data.insert("Byte".to_string(), 34);
        db.save(data.clone()).unwrap();
        info!("DATA: {:?}", db.get().unwrap());
    }
}
