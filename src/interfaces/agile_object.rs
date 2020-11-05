use crate::*;

#[repr(transparent)]
#[derive(Clone, PartialEq)]
pub struct IAgileObject(IUnknown);

unsafe impl ComInterface for IAgileObject {
    type Vtable = IUnknown_vtable;

    const IID: Guid = {
        Guid::from_values(
            0x94EA_2B94,
            0xE9CC,
            0x49E0,
            [0xC0, 0xFF, 0xEE, 0x64, 0xCA, 0x8F, 0x5B, 0x90],
        )
    };
}

unsafe impl GetAbi for IAgileObject {
    type Abi = RawPtr;

    unsafe fn get_abi(&self) -> RawPtr {
        self.0.get_abi()
    }
}