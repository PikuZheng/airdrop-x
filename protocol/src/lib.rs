use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

const MAGIC: &[u8; 4] = b"ADX1";

pub fn send(mut stream: impl Write, path: &Path) -> io::Result<()> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid file name"))?
        .as_bytes();
    let size = path.metadata()?.len();
    let hash = hash_file(path)?;

    stream.write_all(MAGIC)?;
    stream.write_all(&(name.len() as u16).to_be_bytes())?;
    stream.write_all(name)?;
    stream.write_all(&size.to_be_bytes())?;
    stream.write_all(&hash)?;
    io::copy(&mut File::open(path)?, &mut stream)?;
    Ok(())
}

pub fn receive(mut stream: impl Read, directory: &Path) -> io::Result<PathBuf> {
    let mut magic = [0; 4];
    stream.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid protocol",
        ));
    }

    let name_len = read_u16(&mut stream)? as usize;
    if name_len == 0 || name_len > 255 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid file name",
        ));
    }
    let mut name = vec![0; name_len];
    stream.read_exact(&mut name)?;
    let name = Path::new(std::str::from_utf8(&name).map_err(io::Error::other)?)
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "unsafe file name"))?;
    let size = read_u64(&mut stream)?;
    let mut expected_hash = [0; 32];
    stream.read_exact(&mut expected_hash)?;

    std::fs::create_dir_all(directory)?;
    let destination = directory.join(name);
    let temporary = destination.with_extension("airdrop-x.part");
    let mut file = File::create(&temporary)?;
    let copied = io::copy(&mut stream.take(size), &mut file)?;
    drop(file);
    if copied != size || hash_file(&temporary)? != expected_hash {
        let _ = std::fs::remove_file(&temporary);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "incomplete or corrupt file",
        ));
    }
    std::fs::rename(&temporary, &destination)?;
    Ok(destination)
}

fn hash_file(path: &Path) -> io::Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    io::copy(&mut file, &mut hash)?;
    Ok(hash.finalize().into())
}

fn read_u16(reader: &mut impl Read) -> io::Result<u16> {
    let mut bytes = [0; 2];
    reader.read_exact(&mut bytes)?;
    Ok(u16::from_be_bytes(bytes))
}

fn read_u64(reader: &mut impl Read) -> io::Result<u64> {
    let mut bytes = [0; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let root = std::env::temp_dir().join(format!("airdrop-x-{}", std::process::id()));
        let source_dir = root.join("source");
        let target_dir = root.join("target");
        std::fs::create_dir_all(&source_dir).unwrap();
        let source = source_dir.join("hello.txt");
        std::fs::write(&source, b"hello AirDrop-X").unwrap();
        let mut bytes = Vec::new();
        send(&mut bytes, &source).unwrap();
        let received = receive(bytes.as_slice(), &target_dir).unwrap();
        assert_eq!(std::fs::read(received).unwrap(), b"hello AirDrop-X");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_bad_hash() {
        let root = std::env::temp_dir().join(format!("airdrop-x-bad-{}", std::process::id()));
        let source = root.join("hello.txt");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&source, b"hello").unwrap();
        let mut bytes = Vec::new();
        send(&mut bytes, &source).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        assert!(receive(bytes.as_slice(), &root.join("target")).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
