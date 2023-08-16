use std::{ffi::c_void, ops::Deref};

use lean_sys::{
    lean_alloc_external, lean_apply_1, lean_external_class, lean_get_external_class,
    lean_get_external_data, lean_inc, lean_is_exclusive, lean_object, lean_register_external_class,
};

use crate::{Layout, Obj, TObj, TObjRef};

pub trait AsExternalObj: Clone + 'static {
    type ObjIter<'a>: Iterator<Item = &'a Obj>;
    fn obj_iter(&self) -> Self::ObjIter<'_>;

    unsafe extern "C" fn foreach(data: *mut c_void, closure: *mut lean_object) {
        let data = data as *mut Self;
        for i in (*data).obj_iter() {
            lean_inc(i.0);
            lean_inc(closure);
            lean_apply_1(closure, i.0);
        }
    }

    unsafe extern "C" fn finalize(data: *mut c_void) {
        let data = data as *mut Self;
        drop(Box::from_raw(data));
    }

    fn register() -> ExternalClass<Self> {
        let class =
            unsafe { lean_register_external_class(Some(Self::finalize), Some(Self::foreach)) };
        ExternalClass {
            class,
            _type: std::marker::PhantomData,
        }
    }
}

#[repr(transparent)]
pub struct ExternalClass<T: AsExternalObj> {
    class: *mut lean_external_class,
    _type: std::marker::PhantomData<T>,
}

impl<T: AsExternalObj> ExternalClass<T> {
    pub fn class(&self) -> *mut lean_external_class {
        self.class
    }
    pub fn create(&self, data: T) -> TObj<External<T>> {
        let data = Box::new(data);
        let data = Box::into_raw(data);
        let external = External {
            class: self.class,
            data,
        };
        unsafe { TObj::new(External::pack_obj(external)) }
    }
}

pub struct External<T: AsExternalObj> {
    class: *mut lean_external_class,
    data: *mut T,
}

impl<T: AsExternalObj> Layout for External<T> {
    unsafe fn pack_obj(value: Self) -> Obj {
        unsafe { Obj(lean_alloc_external(value.class, value.data as *mut c_void)) }
    }

    unsafe fn unpack_obj(value: Obj) -> Self {
        let class = lean_get_external_class(value.0);
        let data = lean_get_external_data(value.0) as *mut T;
        Self { class, data }
    }
}

impl<T: AsExternalObj> Deref for TObj<External<T>> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(lean_get_external_data(self.obj.0) as *mut T) }
    }
}

impl<T: AsExternalObj> Deref for TObjRef<'_, External<T>> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(lean_get_external_data(self.obj) as *mut T) }
    }
}

impl<T: AsExternalObj> TObj<External<T>> {
    pub fn mutate<F: for<'a> FnOnce(&'a mut T)>(self, class: &ExternalClass<T>, f: F) -> Self {
        unsafe {
            if lean_is_exclusive(self.obj.0) {
                f(&mut *(lean_get_external_data(self.obj.0) as *mut T));
                self
            } else {
                let cloned = class.create((*self).clone());
                f(&mut *(lean_get_external_data(cloned.obj.0) as *mut T));
                cloned
            }
        }
    }
}
