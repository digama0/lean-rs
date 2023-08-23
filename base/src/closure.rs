#![allow(clippy::too_many_arguments)]

use crate::{Layout, Obj, TObj};

impl<R: Layout> TObj<fn() -> R> {
    pub fn invoke(self) -> R {
        let f = self.obj.into_raw();
        unsafe {
            let res = lean_sys::lean_apply_1(f, lean_sys::lean_box(0));
            R::unpack_obj(Obj(res))
        }
    }
}

#[cfg(feature = "nightly")]
impl<R: Layout> FnOnce<()> for TObj<fn() -> R> {
    type Output = R;
    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        self.invoke()
    }
}

macro_rules! fixed_closures {
    (@fn $apply_func:ident($($arg:ident),*)) => {
        #[allow(non_snake_case)]
        impl<$($arg: Layout,)* R: Layout> TObj<fn($($arg,)*) -> R> {
            pub fn invoke(self, $($arg: $arg,)*) -> R {
                let f = self.obj.into_raw();
                unsafe {
                    let res = lean_sys::$apply_func(f, $($arg.pack().into_raw(),)*);
                    R::unpack_obj(Obj(res))
                }
            }
        }

        #[cfg(feature = "nightly")]
        #[allow(non_snake_case)]
        impl<$($arg: Layout,)* R: Layout> FnOnce<($($arg,)*)> for TObj<fn($($arg,)*) -> R> {
            type Output = R;
            extern "rust-call" fn call_once(self, ($($arg,)*): ($($arg,)*)) -> Self::Output {
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
macro_rules! __lean_closure_function {
    ( $ret_type:ty, $body:block, () ) => {
        unsafe extern "C" fn __lean_closure_function(_ : *mut lean_sys::lean_object) -> $crate::Obj {
            let ret_val : $ret_type = $body;
            <$ret_type as  $crate::Layout>::pack_obj(ret_val)
        }
    };
    ( $ret_type:ty, $body:block, ($($arg_name:ident : $arg_type:ty),+ $(,)? ) ) => {
        unsafe extern "C" fn __lean_closure_function(
                $($arg_name : *mut lean_sys::lean_object),+
        ) -> $crate::Obj {
            $(let $arg_name = <$arg_type as $crate::Layout>::unpack_obj($crate::Obj($arg_name));)+
            let ret_val : $ret_type = $body;
            <$ret_type as $crate::Layout>::pack_obj(ret_val)
        }
    };
}
#[macro_export]
macro_rules! lean_closure {
    ( [ $($cap_name:ident : $cap_type:ty),* $(,)? ] || -> $ret_type:ty $body:block ) => {
        $crate::lean_closure!(
            [ $($cap_name : $cap_type),* ] | | -> $ret_type $body
        )
    };
    ( [ $($cap_name:ident : $cap_type:ty),* $(,)? ] | $($arg_name:ident : $arg_type:ty),* $(,)? | -> $ret_type:ty $body:block ) => {
        {
            $crate::__lean_closure_function!($ret_type, $body, ($($cap_name : $cap_type,)* $($arg_name : $arg_type,)*));
            const __ARG : usize = ($crate::__count!($($arg_name)*));
            const __FIXED : usize = $crate::__count!($($cap_name)*);
            const __ARITY : usize = if __ARG == 0 { 1 + __FIXED } else { __ARG + __FIXED };
            unsafe {
                let captures : [$crate::Obj; __FIXED]  = [
                    $(<$cap_type as $crate::Layout>::pack_obj($cap_name),)*
                ];
                let closure = lean_sys::lean_alloc_closure(
                    __lean_closure_function as *mut ::core::ffi::c_void,
                    __ARITY as u32,
                    __FIXED as u32,
                );
                for (idx, arg) in captures.into_iter().enumerate() {
                    lean_sys::lean_closure_set(closure, idx as u32, arg.into_raw());
                }
                $crate::TObj::<fn ($($arg_type,)*)->$ret_type>::unpack_obj($crate::Obj(closure))
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::test::initialize_thread_local_runtime;
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
                        let res: bool = closure.invoke(a, b);
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
                let closure = lean_closure! {
                    [x : bool, y : bool] || -> bool {
                    x || y
                }};
                let res: bool = closure.invoke();
                assert_eq!(res, x || y)
            }
        }
    }
}
