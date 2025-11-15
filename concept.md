## Features der Sprache im Kontext von CPL
- JIT (wenn Zeit ist)
- Starke Typisierung ( typechecking ) ( type inference ) ( generics nur für eingebaute typen )
- Interpreter
- LALRPOP + RUST
- Interfaces, aber keine OOP und Vererbung ( Prozedural + Interfaces, wie in GO)
- Impl Blöcke wie in Rust??
- GC => RefCounting ( alles ist RefCounting, so wie in Swift), heißt man braucht auch irgendwie ein WEAK RC
- 2(4) Stages: Type Inference / Type Checking / Cycle Detection (for warnings) => (optimizing stage) => (compile to bytecode) => running stage (as bytecode)
- https://github.com/TheDan64/inkwell als LLVM JIT Wrapper, https://createlang.rs/01_calculator/jit_intro.html als guide dazu


## Typisierung
- Einfache Typen
- Int, Float, String, Bool, Array(List), (Map)?, Struct (einfaches Gruppieren von Attributen), Function Types, Vielleicht Tupel

- int
- float/double
- string
- bool
- ArrayList / Vector / Vec => keine arrays
- Map

## Kommentare
```c
// Dies ist ein kommentar, Mehrzeilige Kommentare gibt es nicht
```


## Variablen
```go
// Creation
a := 10
b := a

// Assignment
a = 20


// Dynamic typing
a := 10
a = "Hello"

// Lists
a := []
a += e //append e
a -= e //delete e, nur das erste
e := a[0] //access
a[0] = e // einfügen, wenn idx < a.size

// Maps
a := int -> string
a[key] = value

```

## Operators
create: :=
assign: =
equal: == , !=
compare: <,>, <=, >= 
calulate: +, -, *, /
modulo: %


## Flow Control
```c
if (true){
  ...
} else if (...){
  ...
} else{
  ...
}

while (true){
  ...
}
```

## Includes / Use / Import
```js
import module as m
import module2

import ffi "raylib.h" as ray

ray.Vector2
m.<S>
module2.<S>
```

## Struct 
```js
struct <S> {
  a: float, 
  b: string,
  c: bool,
}

component <C> {
  a: float,
  b: string,
  s: S,
  c: bool,
}


impl <S|C> {
  fun f(a: float, b: float): <S|C> {
    
  }
}

impl <I> for <S|C> {
  fun hello_world() {
    println("string")
  }
}
```

## Functions
```js
fun f (a: <type>, b: <type>, c: <type>): <Returntype> => do shit in one line

fun f (a: <type>, b: <type>, c: <type>): <Returntype> {
  do
  shit
  in
  multiple
  lines

  return "is allowed anywhere"

  last value is return
}
```

## Systems
```js

system s1 (a: <P1>, b: <P2>, c: <P3>)
querying
  <P1> as List with {Entity, Component1, Component2 % !Component3}
  <P2> as List with {Entity, Component1, % #Parent: { Component2, Component3}, Any<#Children>{Component4}, Component6 && Component7 && !Component8 || Component9}
  <P3> as Single with {Entity, Camera % MainCamera} {
}

system s2 (a: <P1>, b: <P2>, c: <P3>)
querying
  <P1> as List with {Entity, mut Component1, Component2 % !Component3}
  <P3> as Single with {Entity, Camera % MainCamera} {

    run system s_Not_registered once; // Darf nicht als Gameloop system registriert sein
}


group PreUpdate {
  s1, s2, s3
}

group Update {
  s4,
  s5 -> s6,
  s6 -> s7,
  s6 -> s8,
}


register group PreUpdate -> group Update;

  register s1 -> s2 -> s3;
  register s4 after s1; // s1 -> s4 -> s2 -> s3
  register s5 before s1; // s5 -> s1 -> s4 -> s2 -> s3
  unregister s2; // s5 -> s1 -> s4 -> s3


system s() {
  // Add create (add) entity command to command queue, that is executed AFTER s, and creates and adds an entity to the world state
  create entity e1
    with
      C1,
      C2,
      C3;

  e1 += CameraControllerComponent {
    fov: 45.0,
    speed: 10.0,
    distanceToTarget: 10.0,
  };

  e1 -= CameraControllerComponent;


  trigger TakesDamage on e1;
  trigger global GameOverEvent;

  trigger t1(event: TakesDamage, ...query_abc) {
    event.target == some entity
  }

  trigger t2(event: GameOverEvent, ...query_abc) {
    event.target == null
  }

  // Add remove command to command queue, that removes the entity e1 after s is finished
  remove entity e1
}

```


## Fragen an Gips
