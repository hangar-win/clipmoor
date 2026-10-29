use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, bail};
use serde::Serialize;
use zeroize::Zeroizing;
use windows::Win32::Foundation::{HLOCAL, LocalFree, NTE_BAD_KEYSET};
use windows::Win32::Security::Cryptography::{BCRYPT_OAEP_PADDING_INFO, BCRYPT_RSA_ALGORITHM, BCRYPT_SHA256_ALGORITHM, CERT_KEY_SPEC, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData, MS_PLATFORM_CRYPTO_PROVIDER, NCRYPT_KEY_HANDLE, NCRYPT_LENGTH_PROPERTY, NCRYPT_PAD_OAEP_FLAG, NCRYPT_PROV_HANDLE, NCRYPT_SILENT_FLAG, NCryptCreatePersistedKey, NCryptDecrypt, NCryptDeleteKey, NCryptEncrypt, NCryptFinalizeKey, NCryptOpenKey, NCryptOpenStorageProvider, NCryptSetProperty};
use windows::core::{Owned, PCWSTR, w};

use crate::crypto::MasterKey;

fn blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
    CRYPT_INTEGER_BLOB { cbData: data.len() as u32, pbData: data.as_ptr() as *mut u8 }
}

fn take_blob(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
    unsafe {
        let v = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        std::ptr::write_bytes(out.pbData, 0, out.cbData as usize);
        LocalFree(Some(HLOCAL(out.pbData as _)));
        v
    }
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WrapMethod {
    Tpm,
    Dpapi,
}

pub fn load_or_create(path: &Path, on_first_run: impl FnOnce()) -> anyhow::Result<(MasterKey, WrapMethod)> {
    match fs::read(path) {
        Ok(file) => {
            let (tag, wrapped) = file.split_first().context("master.key is empty")?;
            let (plain, method) = match tag {
                1 => (tpm_unwrap(wrapped)?, WrapMethod::Tpm),
                2 => (dpapi_unwrap(wrapped)?, WrapMethod::Dpapi),
                t => bail!("master.key has unknown wrap method {t}"),
            };
            let plain = Zeroizing::new(plain);
            Ok((MasterKey::from_bytes(plain.as_slice().try_into()?), method))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let master = MasterKey::generate();
            let (tag, wrapped, method) = match tpm_wrap_with(master.as_bytes(), on_first_run) {
                Ok(w) => (1u8, w, WrapMethod::Tpm),
                Err(_) => (2u8, dpapi_wrap(master.as_bytes())?, WrapMethod::Dpapi),
            };
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }
            let tmp = path.with_extension("key.tmp");
            fs::write(&tmp, [&[tag][..], &wrapped].concat())?;
            fs::rename(&tmp, path)?;
            Ok((master, method))
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(not(test))]
const KEY_NAME: PCWSTR = w!("app.clipmoor.master");
#[cfg(test)]
const KEY_NAME: PCWSTR = w!("app.clipmoor.master.test");

fn provider() -> windows::core::Result<Owned<NCRYPT_PROV_HANDLE>> {
    let mut prov = unsafe { Owned::new(NCRYPT_PROV_HANDLE::default()) };
    unsafe { NCryptOpenStorageProvider(&mut *prov, MS_PLATFORM_CRYPTO_PROVIDER, 0)? };
    Ok(prov)
}

fn tpm_key(on_create: impl FnOnce()) -> windows::core::Result<Owned<NCRYPT_KEY_HANDLE>> {
    let prov = provider()?;
    let mut key = unsafe { Owned::new(NCRYPT_KEY_HANDLE::default()) };
    match unsafe { NCryptOpenKey(*prov, &mut *key, KEY_NAME, CERT_KEY_SPEC(0), NCRYPT_SILENT_FLAG) } {
        Ok(()) => return Ok(key),
        Err(e) if e.code() != NTE_BAD_KEYSET => return Err(e),
        Err(_) => {}
    }
    on_create();
    unsafe {
        NCryptCreatePersistedKey(*prov, &mut *key, BCRYPT_RSA_ALGORITHM, KEY_NAME, CERT_KEY_SPEC(0), NCRYPT_SILENT_FLAG)?;
        NCryptSetProperty((*key).into(), NCRYPT_LENGTH_PROPERTY, &2048u32.to_le_bytes(), NCRYPT_SILENT_FLAG)?;
        NCryptFinalizeKey(*key, NCRYPT_SILENT_FLAG)?;
    }
    Ok(key)
}

fn oaep() -> BCRYPT_OAEP_PADDING_INFO {
    BCRYPT_OAEP_PADDING_INFO { pszAlgId: BCRYPT_SHA256_ALGORITHM, ..Default::default() }
}

fn tpm_wrap_with(plain: &[u8], on_create: impl FnOnce()) -> windows::core::Result<Vec<u8>> {
    let key = tpm_key(on_create)?;
    let pad = oaep();
    let pad = Some(&pad as *const _ as *const core::ffi::c_void);
    let mut len = 0u32;
    unsafe { NCryptEncrypt(*key, Some(plain), pad, None, &mut len, NCRYPT_PAD_OAEP_FLAG | NCRYPT_SILENT_FLAG)? };
    let mut out = vec![0u8; len as usize];
    unsafe { NCryptEncrypt(*key, Some(plain), pad, Some(&mut out), &mut len, NCRYPT_PAD_OAEP_FLAG | NCRYPT_SILENT_FLAG)? };
    out.truncate(len as usize);
    Ok(out)
}

pub fn delete_tpm_key() -> windows::core::Result<()> {
    let prov = provider()?;
    let mut key = unsafe { Owned::new(NCRYPT_KEY_HANDLE::default()) };
    match unsafe { NCryptOpenKey(*prov, &mut *key, KEY_NAME, CERT_KEY_SPEC(0), NCRYPT_SILENT_FLAG) } {
        Ok(()) => unsafe { NCryptDeleteKey(std::mem::take(&mut *key), 0) },
        Err(e) if e.code() == NTE_BAD_KEYSET => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn tpm_unwrap(wrapped: &[u8]) -> windows::core::Result<Vec<u8>> {
    let prov = provider()?;
    let mut key = unsafe { Owned::new(NCRYPT_KEY_HANDLE::default()) };
    unsafe { NCryptOpenKey(*prov, &mut *key, KEY_NAME, CERT_KEY_SPEC(0), NCRYPT_SILENT_FLAG)? };
    let pad = oaep();
    let mut out = vec![0u8; wrapped.len()];
    let mut len = 0u32;
    unsafe { NCryptDecrypt(*key, Some(wrapped), Some(&pad as *const _ as *const core::ffi::c_void), Some(&mut out), &mut len, NCRYPT_PAD_OAEP_FLAG | NCRYPT_SILENT_FLAG)? };
    out.truncate(len as usize);
    Ok(out)
}

pub fn dpapi_wrap(plain: &[u8]) -> windows::core::Result<Vec<u8>> {
    let mut out = CRYPT_INTEGER_BLOB::default();
    unsafe { CryptProtectData(&blob(plain), PCWSTR::null(), None, None, None, CRYPTPROTECT_UI_FORBIDDEN, &mut out)? };
    Ok(take_blob(out))
}

pub fn dpapi_unwrap(wrapped: &[u8]) -> windows::core::Result<Vec<u8>> {
    let mut out = CRYPT_INTEGER_BLOB::default();
    unsafe { CryptUnprotectData(&blob(wrapped), None, None, None, None, CRYPTPROTECT_UI_FORBIDDEN, &mut out)? };
    Ok(take_blob(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    fn tpm_lock() -> MutexGuard<'static, ()> {
        static M: Mutex<()> = Mutex::new(());
        M.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn delete_test_key() {
        delete_tpm_key().unwrap();
    }

    #[test]
    fn dpapi_roundtrip() {
        let t = std::time::Instant::now();
        let b = dpapi_wrap(&[5u8; 64]).unwrap();
        let wrapped = t.elapsed();
        let t = std::time::Instant::now();
        assert_eq!(dpapi_unwrap(&b).unwrap(), vec![5u8; 64]);
        println!("dpapi wrap {:?}, unwrap {:?}", wrapped, t.elapsed());
    }

    #[test]
    fn load_or_create_is_stable() {
        let _g = tpm_lock();
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("master.key");
        let (a, method) = load_or_create(&p, || {}).unwrap();
        let (b, again) = load_or_create(&p, || panic!("ikinci açılış oluşturmamalı")).unwrap();
        assert_eq!(a.as_bytes(), b.as_bytes());
        assert_eq!(method, again);
        if method == WrapMethod::Tpm {
            delete_test_key();
        }
    }

    #[test]
    #[ignore]
    fn tpm_roundtrip_and_timing() {
        let _g = tpm_lock();
        delete_test_key();
        let t = std::time::Instant::now();
        let b = tpm_wrap_with(&[7u8; 64], || {}).unwrap();
        let created = t.elapsed();
        let t = std::time::Instant::now();
        assert_eq!(tpm_unwrap(&b).unwrap(), vec![7u8; 64]);
        println!("tpm create+wrap {:?}, open+unwrap {:?}", created, t.elapsed());
        delete_test_key();
    }
}
