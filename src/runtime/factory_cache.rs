use crate::*;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicPtr, Ordering};

type DllGetActivationFactory = extern "system" fn(name: RawPtr, factory: *mut RawPtr) -> ErrorCode;

/// Attempts to load and cache the factory interface for the given WinRT class.
pub struct FactoryCache<C, I> {
    shared: AtomicPtr<std::ffi::c_void>,
    _c: PhantomData<C>,
    _i: PhantomData<I>,
}

impl<C, I> FactoryCache<C, I> {
    pub const fn new() -> Self {
        Self {
            shared: AtomicPtr::new(::std::ptr::null_mut()),
            _c: PhantomData,
            _i: PhantomData,
        }
    }
}

impl<C: RuntimeName, I: ComInterface> FactoryCache<C, I> {
    pub fn call<R, F: FnOnce(&I) -> Result<R>>(&mut self, callback: F) -> Result<R> {
        loop {
            // Attempt to load a previously cached factory pointer.
            let ptr = self.shared.load(Ordering::Relaxed);

            // If a pointer is found, the cache is primed and we're good to go.
            if !ptr.is_null() {
                return callback(unsafe { std::mem::transmute(&ptr) });
            }

            // Otherwise, we load the factory the usual way.
            let factory = factory::<C, I>()?;

            // If the factory is agile, we can safely cache it.
            if factory.is_agile() {
                if self
                    .shared
                    .compare_and_swap(
                        std::ptr::null_mut(),
                        unsafe { std::mem::transmute_copy(&factory) },
                        Ordering::Relaxed,
                    )
                    .is_null()
                {
                    std::mem::forget(factory);
                }
            } else {
                // Otherwise, for non-agile factories we simply use the factory
                // and discard after use as it is not safe to cache.
                return callback(&factory);
            }
        }
    }
}

/// Attempts to load the factory interface for the given WinRT class.
pub fn factory<C: RuntimeName, I: ComInterface>() -> Result<I> {
    let mut factory: Option<I> = None;
    let name = HString::from(C::NAME);

    unsafe {
        // First attempt to get the activation factory via the OS.
        let code = RoGetActivationFactory(name.get_abi(), &I::IID, factory.set_abi());

        // Treat any delay-load errors like standard errors, so that the heuristics
        // below can still load registration-free libraries on Windows versions below 10.
        let mut code = code.unwrap_or_else(|code| code);

        // If this fails because combase hasn't been loaded yet then load combase
        // automatically so that it "just works" for apartment-agnostic code.
        if code == ErrorCode::CO_E_NOTINITIALIZED {
            let mut _cookie = std::ptr::null_mut();

            // Won't get any delay-load errors here if we got CO_E_NOTINITIALIZED, so quiet the
            // warning from the #[must_use] on the returned Result<>.
            let _ = CoIncrementMTAUsage(&mut _cookie);

            // Now try a second time to get the activation factory via the OS.
            code = RoGetActivationFactory(name.get_abi(), &I::IID, factory.set_abi())
                .unwrap_or_else(|code| code);
        }

        // If this succeeded then return the resulting factory interface.
        if code.is_ok() {
            return code.and_some(factory);
        }

        // If not, first capture the error information from the failure above so that we
        // can ultimately return this error information if all else fails.
        let original: Error = code.into();

        // Now attempt to find the factory's implementation heuristically.
        let mut path = C::NAME;

        // Remove the suffix until a match is found. For example, if the class name is
        // "A.B.TypeName" then this will first attempt to load "A.B.dll" and if that
        // fails it will attempt "A.dll" before giving up.
        while let Some(pos) = path.rfind('.') {
            path = &path[..pos];

            // Turn the resulting namespace portion into a DLL name.
            let path: Vec<u16> = path.encode_utf16().chain(".dll\0".encode_utf16()).collect();

            // Attempt to load the DLL.
            let library = Library(LoadLibraryExW(path.as_ptr(), std::ptr::null_mut(), 0));

            if library.0.is_null() {
                continue;
            }

            // If the DLL was found then get the export used to retrieve the factory.
            let library_call = GetProcAddress(library.0, b"DllGetActivationFactory\0".as_ptr());

            if library_call.is_null() {
                continue;
            }

            let library_call: DllGetActivationFactory = std::mem::transmute(library_call);
            let mut abi = std::ptr::null_mut();
            library_call(name.get_abi(), &mut abi);

            if abi.is_null() {
                continue;
            }

            let factory: IActivationFactory = std::mem::transmute(abi);

            // If we get this far it means the factory has been loaded and will be returned
            // to the caller. At this point we need to pin the library to avoid it unloading
            // while there are outstanding references. Unloading is only supported for
            // components loaded via RoGetActivationFactory.
            std::mem::forget(library);
            return factory.query();
        }

        Err(original)
    }
}

struct Library(RawPtr);

impl Drop for Library {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                FreeLibrary(self.0);
            }
        }
    }
}
