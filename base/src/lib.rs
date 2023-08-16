#![allow(clippy::missing_safety_doc)]

pub mod typed;
use lean_sys::*;
pub use typed::*;

pub type ObjPtr = *mut lean_object;
#[repr(transparent)]
pub struct Obj(pub ObjPtr);
impl Drop for Obj {
    fn drop(&mut self) {
        unsafe { lean_dec(self.0) }
    }
}
impl Clone for Obj {
    fn clone(&self) -> Self {
        unsafe { lean_inc(self.0) };
        Self(self.0)
    }
}
impl Obj {
    pub const fn from_usize(n: usize) -> Self {
        Self(lean_box(n))
    }

    pub const fn from_bool(n: bool) -> Self {
        Self(lean_box(n as usize))
    }

    pub unsafe fn ctor_get(&self, i: u32) -> &Self {
        debug_assert!(i < lean_ctor_num_objs(self.0));
        &*lean_ctor_obj_cptr(self.0).add(i as usize).cast()
    }

    pub unsafe fn ctor_tag(&self) -> u32 {
        lean_obj_tag(self.0)
    }

    pub fn into_raw(self) -> ObjPtr {
        let p = self.0;
        std::mem::forget(self);
        p
    }

    pub unsafe fn ctor<const N: usize, T>(tag: u8, args: [Obj; N], scalars: T) -> Obj {
        let o = Obj(lean_alloc_ctor(
            tag.into(),
            args.len() as u32,
            std::mem::size_of::<T>() as _,
        ));
        let ptr = lean_ctor_obj_cptr(o.0).cast::<[Obj; N]>();
        ptr.write(args);
        ptr.add(1).cast::<T>().write(scalars);
        o
    }

    pub fn mk_string(s: &str) -> Self {
        unsafe { Self(lean_mk_string_from_bytes(s.as_ptr(), s.len())) }
    }

    pub unsafe fn to_string(&self) -> &str {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(
            &*lean_string_cstr(self.0),
            lean_string_size(self.0) - 1,
        ))
    }
}

pub const NIL: ObjPtr = lean_box(0);

pub unsafe fn io<T: ?Sized>(res: TObj<IoResult<T>>) -> Result<TObj<T>, Obj> {
    match res.clone().unpack() {
        IoResult::Ok(val) => Ok(val),
        IoResult::Err(res) => Err(res),
    }
}

pub unsafe fn run(
    init: Option<unsafe extern "C" fn(builtin: u8, w: ObjPtr) -> TObj<IoResult<()>>>,
    body: impl FnOnce() -> Result<(), Obj>,
) -> Result<(), Obj> {
    lean_initialize();
    let res;
    if let Some(init) = init {
        lean_set_panic_messages(false);
        res = io(init(1, NIL));
        lean_set_panic_messages(true);
    } else {
        res = Ok(TObj::from_raw(NIL))
    }
    lean_io_mark_end_initialization();
    let res = res.and_then(|_| {
        lean_init_task_manager();
        body()
    });
    lean_finalize_task_manager();
    res
}
