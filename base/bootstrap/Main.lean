import Bootstrap

def main (args : List String) : IO Unit := do
  match foo (Lean.Environment.mk {} {} {} {} {}) with
  | .ok _ => panic! "ho"
  | .error _ => panic! "hi"
