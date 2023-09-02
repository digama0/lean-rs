import Lake
open Lake DSL

require «nest-core» from git "https://github.com/hargonix/nest-core" @ "main"
require «nest-unit» from git "https://github.com/hargonix/nest-unit" @ "main"

package «lean-tests» {
  -- add package configuration options here
}

lean_lib LeanTests {
  precompileModules := true
  -- add library configuration options here
}

@[default_target]
lean_exe «lean-tests» {
  root := `Main
}

partial def enumRustSrc (dir: FilePath) : StateT (Array FilePath) IO Unit := do
  for file in (← dir.readDir) do
    if (← file.path.isDir) then
      enumRustSrc file.path
    else if file.path.extension == "rs" || file.path.extension == "toml"  then
      modify (·.push file.path)

partial def getInputFiles (root: FilePath) : SchedulerM (Array (BuildJob FilePath)) := do
  let files ← enumRustSrc root |>.run #[] |>.catchExceptions (fun _ => default)
  let files := files.snd
  let mut jobs := #[]
  for file in files do
    let file ← inputFile file
    jobs := jobs.push file
  pure jobs

extern_lib liblean_tests pkg := do
  let name := nameToStaticLib "lean_tests"
  let libFile := pkg.buildDir / "lib" / name
  let inputFiles ← getInputFiles pkg.rootDir
  buildFileAfterDepArray libFile inputFiles (fun _ => proc {
    cmd := "cargo",
    args := #["build", "--release", "-Zunstable-options", "--target-dir", (pkg.buildDir / "rust").toString, "--out-dir", (pkg.buildDir / "lib").toString]
  } true) (pure BuildTrace.nil)
