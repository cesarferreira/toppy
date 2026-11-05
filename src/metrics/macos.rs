//! macOS mount sources, used to group APFS volumes that share a physical disk.

use std::collections::HashMap;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

use super::io::physical_disk_id;

/// Mount point to physical disk id (`disk3`), from `getmntinfo`.
pub fn physical_ids_by_mount() -> HashMap<PathBuf, String> {
    let mut map = HashMap::new();
    unsafe {
        let mut buf: *mut libc::statfs = std::ptr::null_mut();
        let count = libc::getmntinfo(&mut buf, libc::MNT_NOWAIT);
        if count <= 0 || buf.is_null() {
            return map;
        }
        for index in 0..count {
            let stat = &*buf.add(index as usize);
            let mount = path_from_c_chars(&stat.f_mntonname);
            let source = string_from_c_chars(&stat.f_mntfromname);
            map.insert(mount, physical_disk_id(&source).to_string());
        }
    }
    map
}

fn bytes_before_nul(bytes: &[libc::c_char]) -> &[u8] {
    let raw = unsafe { std::slice::from_raw_parts(bytes.as_ptr().cast::<u8>(), bytes.len()) };
    let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
    &raw[..end]
}

fn path_from_c_chars(bytes: &[libc::c_char]) -> PathBuf {
    PathBuf::from(OsString::from_vec(bytes_before_nul(bytes).to_vec()))
}

fn string_from_c_chars(bytes: &[libc::c_char]) -> String {
    let raw = bytes_before_nul(bytes);
    String::from_utf8(raw.to_vec())
        .unwrap_or_else(|err| String::from_utf8_lossy(&err.into_bytes()).into_owned())
}
