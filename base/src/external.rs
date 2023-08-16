use std::{ffi::c_void, ops::Deref};

use lean_sys::{
    lean_alloc_external, lean_apply_1, lean_external_class, lean_get_external_data,
    lean_is_exclusive, lean_register_external_class,
};

use crate::{Layout, Obj, ObjPtr, TObj, TObjRef};

pub unsafe trait ForeachObj {
    fn foreach_obj<F: Fn(Obj)>(&self, _f: &F) {}
}

unsafe impl ForeachObj for Obj {
    fn foreach_obj<F: Fn(Obj)>(&self, f: &F) {
        f(self.clone())
    }
}
unsafe impl<A: ?Sized> ForeachObj for TObj<A> {
    fn foreach_obj<F: Fn(Obj)>(&self, f: &F) {
        self.obj.foreach_obj(f)
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ExternalClass<T: ?Sized> {
    class: lean_external_class,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ForeachObj> Default for ExternalClass<T> {
    fn default() -> Self {
        Self::CLASS
    }
}

impl<T: ForeachObj> ExternalClass<T> {
    pub const CLASS: Self = {
        unsafe extern "C" fn foreach<T: ForeachObj>(data: *mut c_void, closure: ObjPtr) {
            let data = data as *mut T;
            let f = TObjRef::<fn(Obj)>::from_raw(closure);
            (*data).foreach_obj(&move |obj| {
                lean_apply_1(f.to_owned().into_raw(), obj.into_raw());
            })
        }

        unsafe extern "C" fn finalize<T>(data: *mut c_void) {
            let data = data as *mut T;
            drop(Box::from_raw(data));
        }

        Self {
            class: lean_external_class {
                m_finalize: Some(finalize::<T>),
                m_foreach: Some(foreach::<T>),
            },
            _phantom: std::marker::PhantomData,
        }
    };

    pub const fn raw(&self) -> *mut lean_external_class {
        &self.class as *const _ as *mut _
    }

    pub fn register(self) -> &'static Self {
        let class =
            unsafe { lean_register_external_class(self.class.m_finalize, self.class.m_foreach) };
        unsafe { &*(class as *mut Self) }
    }
}

pub trait AsExternalObj: ForeachObj + Sized + 'static {}

pub struct External<T: AsExternalObj> {
    data: *mut T,
}

impl<T: AsExternalObj> External<T> {
    pub fn new(data: T) -> Self {
        External {
            data: Box::into_raw(Box::new(data)),
        }
    }
}

impl<T: AsExternalObj> Layout for External<T> {
    unsafe fn pack_obj(value: Self) -> Obj {
        let cls = ExternalClass::<T>::CLASS.raw();
        unsafe { Obj(lean_alloc_external(cls, value.data as *mut c_void)) }
    }

    unsafe fn unpack_obj(value: Obj) -> Self {
        let data = lean_get_external_data(value.0) as *mut T;
        Self { data }
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
        External::new(value).pack()
    }
}

impl<T: AsExternalObj> TObj<External<T>> {
    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
    {
        if !unsafe { lean_is_exclusive(self.obj.0) } {
            *self = (**self).clone().into();
        }
        unsafe { &mut *(lean_get_external_data(self.obj.0) as *mut T) }
    }
}
