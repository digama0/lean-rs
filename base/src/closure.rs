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

#[macro_export]
macro_rules! __count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + $crate::__count!($($xs)*));
}

#[macro_export]
macro_rules! lean_closure {
    ( [ $($cap_name:ident : $cap_type:ty),* $(,)? ] | | -> $ret_type:ty $body:block )  => {
        $crate::lean_closure!(
            [ $($cap_name : $cap_type),* ] | _unit : () | -> $ret_type $body
        )
    };
    ( [ $($cap_name:ident : $cap_type:ty),* $(,)? ] || -> $ret_type:ty $body:block ) => {
        $crate::lean_closure!(
            [ $($cap_name : $cap_type),* ] | _unit : () | -> $ret_type $body
        )
    };
    ( [ $($cap_name:ident : $cap_type:ty),* $(,)? ] -> $ret_type:ty $body:block ) => {
        $crate::lean_closure!(
            [ $($cap_name : $cap_type),* ] | _unit : () | -> $ret_type $body
        )
    };
    ( [ $($cap_name:ident : $cap_type:ty),* $(,)? ] | $($arg_name:ident : $arg_type:ty),+ $(,)? | -> $ret_type:ty $body:block ) => {
        {
            // TODO: maybe the above closure should also use __Convert?
            trait __Convert {
                type Output;
                fn convert(_ : $crate::Obj) -> Self::Output;
                fn capture(_ : Self) -> $crate::Obj;
            }
            impl __Convert for $crate::Obj {
                type Output = $crate::Obj;
                fn convert(x : $crate::Obj) -> Self::Output {
                    x
                }
                fn capture(x : Self) -> Self {
                    x
                }
            }
            impl<T> __Convert for $crate::TObj<T> {
                type Output = $crate::TObj<T>;
                fn convert(x : $crate::Obj) -> Self::Output {
                    unsafe { $crate::TObj::from_raw(x.into_raw()) }
                }
                fn capture(x : Self) -> $crate::Obj {
                    x.into_obj()
                }
            }
            impl<T : $crate::Layout> __Convert for T {
                type Output = T;
                fn convert(x : $crate::Obj) -> Self::Output {
                    unsafe { $crate::TObj::<T>::from_raw(x.into_raw()).unpack() }
                }
                fn capture(x : Self) -> $crate::Obj {
                    x.pack().into_obj()
                }
            }
            extern "C" fn __lean_plain_closure($($cap_name : *mut lean_sys::lean_object,)* $($arg_name : *mut lean_sys::lean_object,)+)
                -> $crate::Obj {
                $(let $cap_name = <$cap_type as __Convert>::convert( $crate::Obj($cap_name));)*
                $(let $arg_name = <$arg_type as __Convert>::convert( $crate::Obj($arg_name));)*
                let ret_val : $ret_type = $body;
                <$ret_type as __Convert>::capture(ret_val)
            }
            const __ARITY : usize = $crate::__count!($($arg_name)*) + $crate::__count!($($cap_name)*);
            const __FIXED : usize = $crate::__count!($($cap_name)*);
            let captures : [$crate::Obj; __FIXED]  = [
                $(<$cap_type as __Convert>::capture($cap_name),)*
            ];
            unsafe {
                let closure = lean_sys::lean_alloc_closure(
                    __lean_plain_closure as *mut ::core::ffi::c_void,
                    __ARITY as u32,
                    __FIXED as u32,
                );
                for (idx, arg) in captures.into_iter().enumerate() {
                    lean_sys::lean_closure_set(closure, idx as u32, arg.into_raw());
                }
                $crate::Obj(closure)
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::{test::initialize_thread_local_runtime, Obj};
    use lean_sys::lean_apply_2;

    use crate::Layout;

    #[test]
    fn boolean_and() {
        initialize_thread_local_runtime();
        for x in [true, false] {
            for y in [true, false] {
                for a in [true, false] {
                    for b in [true, false] {
                        let closure = lean_closure! {
                            [x : bool, y : bool] | a : bool, b : bool | -> bool {
                            x && y && a && b
                        }};
                        let res: bool = unsafe {
                            bool::unpack_obj(Obj(lean_apply_2(
                                closure.into_raw(),
                                a.pack().into_raw(),
                                b.pack().into_raw(),
                            )))
                        };
                        assert_eq!(res, x && y && a && b)
                    }
                }
            }
        }
    }

    #[test]
    fn boolean_or() {
        initialize_thread_local_runtime();
        for x in [true, false] {
            for y in [true, false] {
                for a in [true, false] {
                    for b in [true, false] {
                        let closure = lean_closure! {
                            [x : bool, y : bool] | a : bool, b : bool | -> bool {
                            x || y || a || b
                        }};
                        let res: bool = unsafe {
                            bool::unpack_obj(Obj(lean_apply_2(
                                closure.into_raw(),
                                a.pack().into_raw(),
                                b.pack().into_raw(),
                            )))
                        };
                        assert_eq!(res, x || y || a || b)
                    }
                }
            }
        }
    }
}
