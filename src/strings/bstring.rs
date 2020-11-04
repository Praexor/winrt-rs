use crate::*;

#[repr(C)]
pub struct BString {
    ptr: RawPtr,
}

impl BString {
    pub fn new() -> BString {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        unsafe { SysStringLen(self.ptr) as usize }
    }

    pub fn clear(&mut self) {
        unsafe {
            SysFreeString(self.ptr);
        }

        self.ptr = std::ptr::null_mut();
    }
}

unsafe impl AbiTransferable for BString {
    type Abi = RawPtr;

    unsafe fn get_abi(&self) -> RawPtr {
        self.ptr
    }

    unsafe fn set_abi(&mut self) -> *mut RawPtr {
        self.clear();
        &mut self.ptr
    }
}

impl Drop for BString {
    fn drop(&mut self) {
        self.clear();
    }
}

impl From<BString> for String {
    fn from(from: BString) -> Self {
        if from.is_empty() {
            return String::new();
        }

        unsafe {
            String::from_utf16_lossy(std::slice::from_raw_parts(
                from.ptr as *const u16,
                from.len(),
            ))
        }
    }
}

#[link(name = "oleaut32")]
extern "system" {
    fn SysStringLen(bstr: RawPtr) -> u32;
    fn SysFreeString(bstr: RawPtr);
}
