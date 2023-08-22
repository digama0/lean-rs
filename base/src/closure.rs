#![allow(clippy::too_many_arguments)]

use crate::TObj;

impl<R: ?Sized> TObj<fn() -> R> {
    pub fn invoke(self) -> TObj<R> {
        let f = self.obj.into_raw();
        unsafe {
            let res = lean_sys::lean_apply_1(f, lean_sys::lean_box(0));
            TObj::from_raw(res)
        }
    }
}

#[cfg(feature = "nightly")]
impl<R: ?Sized> FnOnce<()> for TObj<fn() -> R> {
    type Output = TObj<R>;
    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        self.invoke()
    }
}

macro_rules! fixed_closures {
    (@fn $apply_func:ident($($arg:ident),*)) => {
        #[allow(non_snake_case)]
        impl<$($arg: ?Sized,)* R: ?Sized> TObj<fn($($arg,)*) -> R> {
            pub fn invoke(self, $($arg: TObj<$arg>,)*) ->  TObj<R> {
                let f = self.obj.into_raw();
                unsafe {
                    let res = lean_sys::$apply_func(f, $($arg.into_raw(),)*);
                    crate:: TObj::from_raw(res)
                }
            }
        }

        #[cfg(feature = "nightly")]
        #[allow(non_snake_case)]
        impl<$($arg: ?Sized,)* R: ?Sized> FnOnce<($(TObj<$arg>,)*)> for TObj<fn($($arg,)*) -> R> {
            type Output = TObj<R>;
            extern "rust-call" fn call_once(self, ($($arg,)*): ($(TObj<$arg>,)*)) -> Self::Output {
                self.invoke($($arg,)*)
            }
        }
    };
    (@impl ($($args:ident)*)) => {};
    (@impl ($($args:ident)*) ($apply_func:ident $arg:ident) $($rest:tt)*) => {
        fixed_closures!(@fn $apply_func($($args,)* $arg));
        fixed_closures!(@impl ($($args)* $arg) $($rest)*);
    };
    ($($apply_func:ident($arg:ident);)*) => {
        fixed_closures!(@impl () $(($apply_func $arg))*);
    }
}

fixed_closures! {
    lean_apply_1(T1);
    lean_apply_2(T2);
    lean_apply_3(T3);
    lean_apply_4(T4);
    lean_apply_5(T5);
    lean_apply_6(T6);
    lean_apply_7(T7);
    lean_apply_8(T8);
    lean_apply_9(T9);
    lean_apply_10(T10);
    lean_apply_11(T11);
    lean_apply_12(T12);
    lean_apply_13(T13);
    lean_apply_14(T14);
    lean_apply_15(T15);
    lean_apply_16(T16);
}
