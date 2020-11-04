use crate::*;

#[repr(C)]
#[derive(Clone)]
pub struct IActivationFactory(Object);

#[repr(C)]
pub struct IActivationFactory_vtable(
    pub IUnknown_QueryInterface,
    pub IUnknown_AddRef,
    pub IUnknown_Release,
    pub Object_GetIids,
    pub Object_GetRuntimeClassName,
    pub Object_GetTrustLevel,

    pub extern "system" fn(this: RawPtr, object: *mut RawPtr) -> ErrorCode, // ActivateInstance
);

unsafe impl ComInterface for IActivationFactory {
    type Vtable = IActivationFactory_vtable;

    const IID: Guid = {
        Guid::from_values(
            0x0000_0035,
            0x0000,
            0x0000,
            [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
        )
    };
}

unsafe impl AbiTransferable for IActivationFactory {
    type Abi = RawPtr;

    unsafe fn get_abi(&self) -> RawPtr {
        self.0.get_abi()
    }

    unsafe fn set_abi(&mut self) -> *mut RawPtr {
        self.0.set_abi()
    }
}

impl IActivationFactory {
    pub fn activate_instance<I: ComInterface>(&self) -> Result<I> {
        unsafe {
            let mut abi = std::ptr::null_mut();
            (self.vtable().6)(self.get_abi(), &mut abi).ok_ptr(abi)?;
            let object: Object = std::mem::transmute(abi);
            object.query()
    }
}
}

