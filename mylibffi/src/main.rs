use std::{error::Error, os::raw::c_void};

use clang::{Clang, EntityKind, Index};
use libffi::raw::{
    ffi_abi_FFI_DEFAULT_ABI, ffi_call, ffi_cif, ffi_prep_cif, ffi_status_FFI_OK, ffi_type,
    ffi_type_sint32,
};

fn main() -> Result<(), Box<dyn Error>> {
    let clang = Clang::new().unwrap();
        let index = Index::new(&clang, false, false);
        let tu = index.parser("lib/lib.h")
            .parse()
            .unwrap();

        tu.get_entity().visit_children(|child, _| {
            if child.get_kind() == EntityKind::StructDecl {
                if let Some(name) = child.get_name() {
                    println!("struct {}", name);

                    child.visit_children(|field, _| {
                        if field.get_kind() == EntityKind::FieldDecl {
                            println!("  {}: {:?}",
                                     field.get_name().unwrap(),
                                     field.get_type().unwrap().get_display_name());
                        }
                        clang::EntityVisitResult::Continue
                    });
                }
            }
            clang::EntityVisitResult::Continue
        });


    unsafe {
        let lib = libloading::Library::new("./lib/liblib.dylib")?;
        let add_one: libloading::Symbol<unsafe extern "C" fn()> = lib.get(b"add_one")?;

        let mut cif = ffi_cif::default();

        let mut type_int = ffi_type_sint32;
        let mut args = [&mut type_int as *mut ffi_type];

        let status = ffi_prep_cif(
            &mut cif,
            ffi_abi_FFI_DEFAULT_ABI,
            1,
            &mut type_int as *mut ffi_type,
            args.as_mut_ptr(),
        );
        assert_eq!(status, ffi_status_FFI_OK);

        let mut param: i32 = 10;
        let mut params = [&mut param as *mut _ as *mut c_void];
        let mut result: i32 = 0;

        ffi_call(
            &mut cif,
            Some(*add_one),
            &mut result as *mut _ as *mut c_void,
            params.as_mut_ptr(),
        );

        assert_eq!(param + 1, result);

        Ok(())
    }
}
