mod closure;

fn main() {
    {
        let out_path =
            std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("fixed_closures.rs");
        let fixed_closures = closure::prettified_fixed_closures();
        std::fs::write(&out_path, fixed_closures)
            .unwrap_or_else(|_| panic!("Couldn't write to {}", out_path.display()));
    }
}
