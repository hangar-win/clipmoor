#![allow(non_snake_case)]

use std::ffi::c_void;
use std::ptr::null_mut;

use windows::Security::Credentials::{KeyCredential, KeyCredentialCreationOption, KeyCredentialManager, KeyCredentialOperationResult, KeyCredentialRetrievalResult, KeyCredentialStatus};
use windows::Storage::Streams::{DataReader, DataWriter};
use windows::Win32::Foundation::HWND;
use windows::core::{GUID, HRESULT, HSTRING, IUnknown, IUnknown_Vtbl, Interface};
use windows_future::IAsyncOperation;

pub const CHALLENGE: &[u8] = b"app.clipmoor/lock/v1";
pub const KEY_MISSING: &str = "lockKeyMissing";
pub const CANCELED: &str = "lockCanceled";
const NAME: &str = "app.clipmoor";

#[windows_core::interface("30B1B9C9-61EF-43E8-88AC-CC433B38D1A6")]
unsafe trait IKeyCredentialManagerCreateWithWindowStatics: IUnknown {
    fn GetIids(&self, count: *mut u32, iids: *mut *mut GUID) -> HRESULT;
    fn GetRuntimeClassName(&self, name: *mut *mut c_void) -> HRESULT;
    fn GetTrustLevel(&self, level: *mut i32) -> HRESULT;
    fn RequestCreateForWindowAsync(&self, window: u64, name: *mut c_void, option: KeyCredentialCreationOption, value: *mut *mut c_void) -> HRESULT;
}

#[windows_core::interface("050DEB3B-B19C-4635-9DF6-5650D66C62B1")]
unsafe trait IKeyCredentialWithWindow: IUnknown {
    fn GetIids(&self, count: *mut u32, iids: *mut *mut GUID) -> HRESULT;
    fn GetRuntimeClassName(&self, name: *mut *mut c_void) -> HRESULT;
    fn GetTrustLevel(&self, level: *mut i32) -> HRESULT;
    fn RequestSignForWindowAsync(&self, window: u64, data: *mut c_void, value: *mut *mut c_void) -> HRESULT;
}

type Outcome<T> = windows::core::Result<Result<T, KeyCredentialStatus>>;

fn credential(hwnd: HWND, create: bool) -> Outcome<KeyCredential> {
    let name = HSTRING::from(NAME);
    let mut found = KeyCredentialManager::OpenAsync(&name)?.join()?;
    if create && found.Status()? == KeyCredentialStatus::NotFound {
        let statics: IKeyCredentialManagerCreateWithWindowStatics = windows::core::factory::<KeyCredentialManager, _>()?;
        let mut op = null_mut();
        unsafe { statics.RequestCreateForWindowAsync(hwnd.0 as u64, std::mem::transmute_copy(&name), KeyCredentialCreationOption::FailIfExists, &mut op).ok()? };
        found = unsafe { IAsyncOperation::<KeyCredentialRetrievalResult>::from_raw(op) }.join()?;
    }
    Ok(match found.Status()? {
        KeyCredentialStatus::Success => Ok(found.Credential()?),
        s => Err(s),
    })
}

fn sign_raw(hwnd: HWND, create: bool) -> Outcome<Vec<u8>> {
    let cred = match credential(hwnd, create)? {
        Ok(c) => c,
        Err(s) => return Ok(Err(s)),
    };
    let writer = DataWriter::new()?;
    writer.WriteBytes(CHALLENGE)?;
    let data = writer.DetachBuffer()?;
    let signer: IKeyCredentialWithWindow = cred.cast()?;
    let mut op = null_mut();
    unsafe { signer.RequestSignForWindowAsync(hwnd.0 as u64, data.as_raw(), &mut op).ok()? };
    let result = unsafe { IAsyncOperation::<KeyCredentialOperationResult>::from_raw(op) }.join()?;
    let status = result.Status()?;
    if status != KeyCredentialStatus::Success {
        return Ok(Err(status));
    }
    let buf = result.Result()?;
    let mut out = vec![0u8; buf.Length()? as usize];
    DataReader::FromBuffer(&buf)?.ReadBytes(&mut out)?;
    Ok(Ok(out))
}

pub fn delete_credential() -> windows::core::Result<()> {
    let name = HSTRING::from(NAME);
    if KeyCredentialManager::OpenAsync(&name)?.join()?.Status()? == KeyCredentialStatus::NotFound {
        return Ok(());
    }
    KeyCredentialManager::DeleteAsync(&name)?.join()
}

pub fn sign(hwnd: HWND, create: bool) -> Result<Vec<u8>, String> {
    match sign_raw(hwnd, create) {
        Ok(Ok(sig)) => Ok(sig),
        Ok(Err(s)) if s == KeyCredentialStatus::NotFound => Err(KEY_MISSING.into()),
        Ok(Err(s)) if s == KeyCredentialStatus::UserCanceled => Err(CANCELED.into()),
        Ok(Err(s)) => Err(format!("hello status {}", s.0)),
        Err(e) => Err(format!("hello: {e}")),
    }
}
