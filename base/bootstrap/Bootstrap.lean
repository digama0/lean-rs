import Lean
import Lake.Load.Package
open Lean

/-- Unsafe implementation of `evalConstCheck`. -/
unsafe def unsafeEvalConstCheck (env : Environment) (opts : Options) (α) (type : Name) (const : Name) : Except String α :=
  match env.find? const with
  | none => throw s!"unknown constant '{const}'"
  | some info =>
    match info.type with
    | Expr.const c _ =>
      if c != type then
        throwUnexpectedType
      else
        env.evalConst α opts const
    | _ => throwUnexpectedType
where
  throwUnexpectedType : Except String α :=
    throw s!"unexpected type at '{const}', `{type}` expected"

/-- Like `Lean.Environment.evalConstCheck`, but with plain universe-polymorphic `Except`. -/
@[implemented_by unsafeEvalConstCheck] opaque evalConstCheck'
(env : Environment) (opts : Options) (α) (type : Name) (const : Name) : Except String α

/-- Load a `PackageConfig` from a configuration environment. -/
def PackageConfig.loadFromEnv
(env : Environment) (opts := Options.empty) : Except String Lake.PackageConfig := do
  let declName ←
    match Lake.packageAttr.ext.getState env |>.toList with
    | [] => .error s!"configuration file is missing a `package` declaration"
    | [name] => pure name
    | _ => .error s!"configuration file has multiple `package` declarations"
  evalConstCheck' env opts _  ``Lake.PackageConfig declName

def foo := PackageConfig.loadFromEnv
