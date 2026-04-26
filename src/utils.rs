use std::{fs, io::Read, str};

pub(crate) fn for_each_chunk<F>(path: &str, mut f: F) -> crate::DynResult<()>
where
    F: FnMut(&str) -> crate::DynResult<()>,
{
    let mut buf = [0; crate::CHUNK_SIZE];
    let mut len = 0;
    let mut file = fs::File::open(path)?;
    loop {
        match file.read(&mut buf[len..]) {
            Ok(0) if len > 0 => return Err("文件不完整".into()),
            Ok(0) => return Ok(()),
            Ok(n) => len += n,
            Err(e) => return Err(e.into()),
        }
        match str::from_utf8(&buf[..len]) {
            Ok(s) => {
                f(s)?;
                len = 0
            }
            Err(e) if e.error_len().is_none() => unsafe {
                let v = e.valid_up_to();
                f(str::from_utf8_unchecked(&buf[..v]))?;
                buf.copy_within(v..len, 0);
                len -= v
            },
            Err(e) => return Err(e.into()),
        };
    }
}
