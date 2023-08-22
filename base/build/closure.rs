use syn::{parse_quote, ExprField, File, Type};

fn generate_arg_idents(n: usize) -> Box<[syn::Ident]> {
    (0..n)
        .map(|i| syn::Ident::new(&format!("Arg{}", i), proc_macro2::Span::call_site()))
        .collect()
}

fn generate_arg_names(n: usize) -> Box<[syn::Ident]> {
    (0..n)
        .map(|i| syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site()))
        .collect()
}

fn generate_fixed_closure(n: usize) -> File {
    let closure_name = syn::Ident::new(&format!("Closure{}", n), proc_macro2::Span::call_site());
    let arg_idents = generate_arg_idents(n);
    let arg_names = generate_arg_names(n);
    let arg_types: Box<[Type]> = arg_idents
        .iter()
        .map(|x| syn::parse_quote!(crate::TObj<#x>))
        .collect();
    let apply_func = syn::Ident::new(&format!("lean_apply_{}", n), proc_macro2::Span::call_site());
    let arg_exprs: Box<[ExprField]> = (0..n)
        .map(|x| syn::parse_str(&format!("args.{}", x)).unwrap())
        .collect();
    syn::parse_quote! {
        #[derive(Clone)]
        pub struct #closure_name<Output : ?Sized, #(#arg_idents : ?Sized,)*>(
            core::marker::PhantomData<Output>,
            #(core::marker::PhantomData<#arg_idents>,)*
        );

        impl<Output : ?Sized, #(#arg_idents : ?Sized,)*> crate::TObj<#closure_name<Output, #(#arg_idents,)*>> {
            pub fn invoke(self, #(#arg_names : #arg_types,)*) ->  crate::TObj<Output> {
                let f = self.obj.into_raw();
                unsafe {
                    let res = lean_sys::#apply_func(f, #(#arg_names.into_raw(),)*);
                    crate:: TObj::from_raw(res)
                }
            }
        }

        #[cfg(feature = "nightly")]
        impl<Output : ?Sized, #(#arg_idents : ?Sized,)*> FnOnce<(#(#arg_types,)*)> for crate::TObj<#closure_name<Output, #(#arg_idents,)*>> {
            type Output = crate::TObj<Output>;
            extern "rust-call" fn call_once(self, args: (#(#arg_types,)*)) -> Self::Output {
                self.invoke(#(#arg_exprs,)*)
            }
        }
    }
}

pub fn prettified_fixed_closures() -> String {
    let all_closures = (1..=16)
        .map(generate_fixed_closure)
        .reduce(|acc, x| parse_quote!(#acc #x))
        .expect("failed to generated fixed closures");
    prettyplease::unparse(&all_closures)
}
