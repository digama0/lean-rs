use lean_base::*;
use lean_sys::*;

extern "C" {
    fn initialize_Lean(builtin: u8, w: ObjPtr) -> TObj<IoResult<()>>;
    fn l_Lean_findSysroot(_: TObj<str>, w: ObjPtr) -> TObj<IoResult<str>>;
    fn l_Lean_initSearchPath(
        leanSysroot: TObj<str>,
        sp: TObj<List<str>>,
        w: ObjPtr,
    ) -> TObj<IoResult<()>>;
    fn l_String_toName(s: TObj<str>) -> TObj<Name>;
    fn lean_import_modules(
        imports: TObj<List<Import>>,
        opts: TObj<Options>,
        trustLevel: u32,
        w: ObjPtr,
    ) -> TObj<IoResult<Environment>>;
    fn l_Lean_SMap_size___rarg(s: ObjPtr) -> Obj;
    fn lean_eval_const(
        env: TObjRef<'_, Environment>,
        opts: TObjRef<'_, Options>,
        name: TObjRef<'_, Name>,
    ) -> TObj<Except<str, ()>>;
    fn lean_run_frontend(
        input: TObj<str>,
        opts: TObj<Options>,
        filename: TObj<str>,
        main_module: TObj<Name>,
        trust_level: u32,
        ilean_file: TObj<LeanOption<str>>,
        w: ObjPtr,
    ) -> TObj<IoResult<Pair<Environment, bool>>>;
    fn lean_enable_initializer_execution(w: ObjPtr) -> TObj<IoResult<()>>;
}

unsafe fn init() -> Result<(), Obj> {
    let sysroot = io(l_Lean_findSysroot("lean".into(), NIL))?;
    io(l_Lean_initSearchPath(
        sysroot,
        List::Nil.pack(),
        // List::Cons("base/bootstrap/build/lib".into(), List::Nil.pack()).pack(),
        NIL,
    ))?;
    io(lean_enable_initializer_execution(NIL))?;
    Ok(())
}

unsafe fn import_modules(mods: &[&str], opts: TObj<Options>) -> Result<TObj<Environment>, Obj> {
    let mut imports = List::Nil.pack();
    for &mod_ in mods {
        let mod_ = l_String_toName(mod_.into());
        let import = TObj::new(Obj::ctor(0, [mod_.into_obj()], false));
        imports = List::Cons(import, imports).pack();
    }
    io(lean_import_modules(imports, opts, 0, NIL))
}

unsafe fn eval_const<T: ?Sized>(
    env: TObjRef<Environment>,
    opts: TObjRef<Options>,
    decl_name: TObjRef<Name>,
) -> TObj<T> {
    match lean_eval_const(env, opts, decl_name).unpack() {
        Except::Ok(val) => TObj::new(val.into_obj()),
        Except::Err(s) => panic!("{}", &*s),
    }
}
fn run_frontend(
    file: &str,
    opts: TObj<Options>,
    mod_name: TObj<Name>,
) -> Result<TObj<Environment>, Obj> {
    unsafe {
        let contents = std::fs::read_to_string(file).unwrap();
        let (env, ok) = io(lean_run_frontend(
            (&*contents).into(),
            opts,
            file.into(),
            mod_name,
            1024,
            None.pack(),
            NIL,
        ))?
        .unpack();
        assert!(ok.unpack(), "file has errors; aborting");
        Ok(env)
    }
}

unsafe fn body() -> Result<(), Obj> {
    let file = std::env::args().nth(1).unwrap_or_else(|| ".".into()) + "/lakefile.lean";
    init()?;
    let opts = TObj::from_raw(NIL);
    // let env = import_modules(&["Bootstrap"], opts.clone())?;
    // let s = eval_const::<str>(
    //     env.as_ref(),
    //     opts.as_ref(),
    //     l_String_toName(TObj::from("foo")).as_ref(),
    // )?;
    // println!("{}", &*s);

    let env = run_frontend(&file, opts, l_String_toName("_Lakefile".into()))?;
    println!(
        "{} constants",
        lean_unbox(l_Lean_SMap_size___rarg(env.unpack().constants.0).0)
    );
    Ok(())
}

fn main() {
    unsafe {
        // let res = run(Some(initialize_Lean), || body());
        let res = run(None, || body());
        if let Err(res) = res {
            lean_io_result_show_error(IoResult::<()>::Err(res).pack().into_obj().0);
        }
    }
}
