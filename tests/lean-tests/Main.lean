import NestCore
import NestUnit
import «LeanTests»

open Nest.Core
open Nest.Unit

def tests : TestTree := [nest| 
group "Self Tests"
  group "List Sum Tests"
    test "Empty" : UnitTest := do
      assert <| listSum [] == 0
    test "Single" : UnitTest := do
      assert <| listSum [1] == 1
    test "Multiple" : UnitTest := do
      assert <| listSum [1, 3, 5, 7, 9] == 25
]

def main : IO UInt32 := Nest.Core.defaultMain tests
