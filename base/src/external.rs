use std::{ffi::c_void, ops::Deref};

use lean_sys::{
    lean_alloc_external, lean_apply_1, lean_external_class, lean_get_external_class,
    lean_get_external_data, lean_inc, lean_is_exclusive, lean_object,
};

use crate::{Layout, Obj, TObj, TObjRef};

pub unsafe trait Iterable {
    fn foreach<F: Fn(&Obj)>(&self, _f: F) {}
}

pub unsafe trait AsExternalObj: Iterable + Clone + 'static {
    const CLASS: lean_external_class = lean_external_class {
        m_foreach: Some(foreach::<Self>),
        m_finalize: Some(finalize::<Self>),
    };
}

unsafe extern "C" fn foreach<T: Iterable>(data: *mut c_void, closure: *mut lean_object) {
    let data = data as *mut T;
    (*data).foreach(|obj| {
        lean_inc(obj.0);
        lean_inc(closure);
        lean_apply_1(closure, obj.0);
    });
}

unsafe extern "C" fn finalize<T>(data: *mut c_void) {
    let data = data as *mut T;
    drop(Box::from_raw(data));
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

impl<T: AsExternalObj> From<T> for TObj<External<T>> {
    fn from(value: T) -> Self {
        let data = Box::into_raw(Box::new(value));
        let class = &T::CLASS as *const _ as *mut _;
        let external = External { data, class };
        external.pack()
    }
}

impl<T: AsExternalObj> TObj<External<T>> {
    pub fn make_mut(&mut self) -> &mut T {
        if !unsafe { lean_is_exclusive(self.obj.0) } {
            *self = (**self).clone().into();
        }
        unsafe { &mut *(lean_get_external_data(self.obj.0) as *mut T) }
    }
}
