#![allow(clippy::type_complexity, clippy::too_many_arguments)]
#[derive(Clone)]
pub struct Closure0<Output: ?Sized>(core::marker::PhantomData<Output>);

impl<Output: ?Sized> crate::TObj<Closure0<Output>> {
    pub fn invoke(self) -> crate::TObj<Output> {
        let f = self.obj.into_raw();
        unsafe {
            let res = crate::lean_apply_1(f, crate::lean_box(0));
            crate::TObj::from_raw(res)
        }
    }
}

#[cfg(feature = "nightly")]
impl<Output: ?Sized> FnOnce<()> for crate::TObj<Closure0<Output>> {
    type Output = crate::TObj<Output>;
    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        self.invoke()
    }
}

include!(concat!(env!("OUT_DIR"), "/fixed_closures.rs"));
