use core::{marker::PhantomData, ops::Deref};

use super::*;

pub trait Layout {
    unsafe fn pack_obj(_: Self) -> Obj;
    unsafe fn unpack_obj(_: Obj) -> Self;
    fn pack(self) -> TObj<Self>
    where
        Self: Sized,
    {
        unsafe { TObj::new(Self::pack_obj(self)) }
    }
}

#[repr(transparent)]
pub struct TObj<A: ?Sized> {
    pub(crate) obj: Obj,
    val: PhantomData<A>,
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TObjRef<'a, A: ?Sized> {
    obj_ref: ObjRef<'a>,
    val: PhantomData<A>,
}

impl<A: ?Sized> Clone for TObj<A> {
    fn clone(&self) -> Self {
        Self {
            obj: self.obj.clone(),
            val: self.val,
        }
    }
}

impl<A: ?Sized> TObjRef<'_, A> {
    pub fn to_owned(self) -> TObj<A> {
        unsafe { TObj::from_obj(self.obj_ref.to_owned()) }
    }
}

impl<A: ?Sized> TObj<A> {
    pub const unsafe fn new(obj: Obj) -> Self {
        Self {
            obj,
            val: PhantomData,
        }
    }
    pub const unsafe fn from_raw(obj: ObjPtr) -> Self {
        Self::new(Obj(obj))
    }
    pub const unsafe fn from_obj(obj: Obj) -> Self {
        Self::new(obj)
    }
    pub fn into_obj(self) -> Obj {
        self.obj
    }
    pub fn into_raw(self) -> ObjPtr {
        self.obj.into_raw()
    }
    pub fn unpack(self) -> A
    where
        A: Layout + Sized,
    {
        unsafe { A::unpack_obj(self.obj) }
    }
    pub fn as_ref(&self) -> TObjRef<'_, A> {
        TObjRef {
            obj_ref: self.obj.as_ref(),
            val: PhantomData,
        }
    }
}

impl Deref for TObj<str> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe { self.obj.to_string() }
    }
}

impl From<&str> for TObj<str> {
    fn from(value: &str) -> Self {
        unsafe { TObj::new(Obj::mk_string(value)) }
    }
}

pub enum List<A: ?Sized> {
    Nil,
    Cons(TObj<A>, TObj<List<A>>),
}

impl<A: ?Sized> Layout for List<A> {
    unsafe fn pack_obj(layout: Self) -> Obj {
        match layout {
            List::Nil => Obj(NIL),
            List::Cons(hd, tl) => Obj::ctor(1, [hd.obj, tl.obj], ()),
        }
    }

    unsafe fn unpack_obj(o: Obj) -> Self {
        match o.ctor_tag() {
            0 => Self::Nil,
            1 => Self::Cons(
                TObj::new(o.ctor_get(0).clone()),
                TObj::new(o.ctor_get(1).clone()),
            ),
            _ => unreachable!(),
        }
    }
}

pub enum IoResult<T: ?Sized> {
    Ok(TObj<T>),
    Err(Obj),
}

impl<T: ?Sized> Layout for IoResult<T> {
    unsafe fn pack_obj(layout: Self) -> Obj {
        match layout {
            IoResult::Ok(e) => Obj::ctor(0, [e.obj, Obj(NIL)], ()),
            IoResult::Err(e) => Obj::ctor(1, [e, Obj(NIL)], ()),
        }
    }

    unsafe fn unpack_obj(o: Obj) -> Self {
        match lean_ptr_tag(o.0) {
            0 => Self::Ok(TObj::new(o.ctor_get(0).clone())),
            1 => Self::Err(o.ctor_get(0).clone()),
            _ => unreachable!(),
        }
    }
}

pub enum Except<E: ?Sized, T: ?Sized> {
    Err(TObj<E>),
    Ok(TObj<T>),
}

impl<E: ?Sized, T: ?Sized> Layout for Except<E, T> {
    unsafe fn pack_obj(layout: Self) -> Obj {
        match layout {
            Except::Err(e) => Obj::ctor(0, [e.obj], ()),
            Except::Ok(e) => Obj::ctor(1, [e.obj], ()),
        }
    }

    unsafe fn unpack_obj(o: Obj) -> Self {
        match lean_ptr_tag(o.0) {
            0 => Self::Err(TObj::new(o.ctor_get(0).clone())),
            1 => Self::Ok(TObj::new(o.ctor_get(0).clone())),
            _ => unreachable!(),
        }
    }
}

pub type LeanOption<T> = Option<TObj<T>>;

impl<T: ?Sized> Layout for LeanOption<T> {
    unsafe fn pack_obj(layout: Self) -> Obj {
        match layout {
            None => Obj(NIL),
            Some(e) => Obj::ctor(1, [e.obj], ()),
        }
    }

    unsafe fn unpack_obj(o: Obj) -> Self {
        match lean_ptr_tag(o.0) {
            0 => None,
            1 => Some(TObj::new(o.ctor_get(0).clone())),
            _ => unreachable!(),
        }
    }
}

pub type Pair<A, B> = (TObj<A>, TObj<B>);

impl<A: ?Sized, B: ?Sized> Layout for Pair<A, B> {
    unsafe fn pack_obj(layout: Self) -> Obj {
        Obj::ctor(0, [layout.0.obj, layout.1.obj], ())
    }

    unsafe fn unpack_obj(o: Obj) -> Self {
        (
            TObj::new(o.ctor_get(0).clone()),
            TObj::new(o.ctor_get(1).clone()),
        )
    }
}

impl Layout for bool {
    unsafe fn pack_obj(layout: Self) -> Obj {
        Obj(lean_box(layout as usize))
    }

    unsafe fn unpack_obj(o: Obj) -> Self {
        lean_unbox(o.into_raw()) != 0
    }
}

pub struct Environment {
    pub const_to_mod_idx: Obj,
    pub constants: Obj,
    pub extensions: Obj,
    pub extra_const_names: Obj,
    pub header: Obj,
}

impl Layout for Environment {
    unsafe fn pack_obj(
        Environment {
            const_to_mod_idx,
            constants,
            extensions,
            extra_const_names,
            header,
        }: Self,
    ) -> Obj {
        Obj::ctor(
            0,
            [
                const_to_mod_idx,
                constants,
                extensions,
                extra_const_names,
                header,
            ],
            (),
        )
    }

    unsafe fn unpack_obj(o: Obj) -> Self {
        Self {
            const_to_mod_idx: o.ctor_get(0).clone(),
            constants: o.ctor_get(1).clone(),
            extensions: o.ctor_get(2).clone(),
            extra_const_names: o.ctor_get(3).clone(),
            header: o.ctor_get(4).clone(),
        }
    }
}

pub struct Name;
pub struct Module;
pub struct Options;
pub struct Import;
