---
# try also 'default' to start simple
theme: academic
# some information about your slides (markdown enabled)
title: Borrow Checking
info: None
# apply UnoCSS classes to the current slide
class: text-center
# https://sli.dev/features/drawing
drawings:
  persist: false
# slide transition: https://sli.dev/guide/animations.html#slide-transitions
transition: null
# enable MDC Syntax: https://sli.dev/features/mdc
mdc: true
# duration of the presentation
duration: 30min
hideInToc: true
---
# Borrow Checking


---
transition: null
hideInToc: true
---
# Agenda

<Toc maxDepth="1"/>


---
transition: null
---
# Bisher?

- Garbage Collection (Java / Python / JS / TS/ Rubs / Lua)
  - Vereinfachte Entwicklung
  - Speicher wird zur Nebensache
  - Fokus auf höhere Konzepte

<v-click>

- Manual Memory Management (C / C++ / Zig)
  - Starke Performance
  - Hohes Level an Kontrolle
  - Fehleranfällig!
</v-click>

---
transition: null
---
## Wofür überhaupt Performance

- Embedded Entwicklung
  - Eingeschränkter Speicher und Prozessorgeschwindigkeit
- Hochskalierende Datenbanken
  - Postgresql, ScyllaDB, etc.
- Alles mit Low Level Speicherzugriff oder High Performance!


---
transition: null
---
## Häufige Probleme mit manueller Speicherverwaltung

````md magic-move
```c {*|*}{lines:true}
#include "stdlib.h"
#include "stdio.h"

int main()
{
    int *abc = malloc(sizeof(int) * 1);
    *abc = 10;

    printf("%d", *abc);
    return 0;
}
```
```c {9|*|*|9-10}{lines:true}
#include "stdlib.h"
#include "stdio.h"

int main()
{
    int *abc = malloc(sizeof(int) * 1);
    *abc = 10;

    free(abc);
    printf("%d", *abc);
    return 0;
}
```
```c {10}{lines:true}
#include "stdlib.h"
#include "stdio.h"

int main()
{
    int *abc = malloc(sizeof(int) * 1);
    *abc = 10;

    free(abc);
    printf("%d", *abc); // Use after free
    return 0;
}
```
````

<div v-click="[1,3]">

Was fehlt?
</div>

<div v-click="[2,3]">

  - Speicher freigeben
</div>

<div v-click="[4,7]">

Und jetzt? 
</div>

---
transition: null
---
## Resource Acquisition is Initialization (RAII)

- Beim Erzeugen von Ressourcen, werden diese initialisiert
  - Destruktor deinitialisiert wieder


```cpp {*|3,10|4,5|8,9}{lines:true}
  #include <vector>

  {
    // Initialization
    std::vector<int> my_list;


    // Hier endet das Scope
    // Der Speicher innerhalt des Vectors wird freigegeben
  }
```

<v-click>

- Funktioniert nur mit Destruktions-Erkennung und Mechanismus
  - Daher nur in C++ und nicht in C
</v-click>

---
transition: null
---
## Resource Acquisition is Initialization (RAII)

- std::move kann genutzt werden, um Ressourcen zu bewegen, ohne zu kopieren 
  - Invalidiert originale Variable


````md magic-move
```cpp {*|3|4}{lines:true}
  #include <vector>

  std::vector<int> a;
  auto b = std::move(a);
```
```cpp {4}{lines:true}
  #include <vector>

  std::vector<int> a; 
  auto b = std::move(a); // a invalidated. At least in Theory
```
````

<v-click>

- Rule of 5 notwendig
  - Move Constructor (und Move Assignment) müssen implementiert werden
  - Viel händischer Extraaufwand für eigene Datentypen und Objekte
  - Bei fehlendem Move Constructor wir auf Copy Constructor zurückgegriffen
</v-click>

---
transition: null
---
## RAII in C++

````md magic-move
```cpp {*|2|3|5}{lines:true}
/// Use after Move
std::vector<int> foo;
std::vector<int> bar = std::move(foo);

foo.push_back(1); 

```
```cpp {3|5}{lines:true}
/// Use after Move
std::vector<int> foo;
std::vector<int> bar = std::move(foo); // Invalidate `foo`, in theory

foo.push_back(1); // Use after Move, undefined behaviour, gültiger Zustand, aber nicht definiert

```
````

```cpp {*|2|3-6|8}{lines:true}
/// Use after free
std::vector<int> *bar = nullptr; 
{
  std::vector<int> foo;
  bar = &foo;
}

bar.push_back(1); // Object is already freed
```

---
transition: null
---
## RAII in C++

- Ganz nett, aber:
  - Nicht gut in die Sprache integriert
    - Rule of 3/5
    - Händischer std::move Aufruf notwendig
    - Use after move möglich
    - Nicht mit Referencen verknüpft (Lifetimes)
    - Implizites kopieren
    - R-Value, L-Value


---
transition: null
layout: image-right
image: /img/oxide.jpg
---
# Rust Borrow Checker


---
transition: null
---
## Rust Borrow Checker

- Integraler Bestandteil der Programmiersprache Rust
- Prüft Speichervalidität zur Compile-Zeit
  - Weder manuelles Speichermanagement notwendig, noch GC collection zur Laufzeit 
- Memory safety guided by compilation
- Performance vergleichbar mit C/C++, und dazu noch Speichersicher

---
transition: null
---
## Ownership

- Es gibt nur einen Besitzer (Owner)
  - Darf nie verletzt werden!
  - Wenn Owner aus dem Scope läuft, wird der Destruktor aufgerufen
  - Der Owner kann wechseln
  - Wenn Variable aktuell kein Owner ist und verwendet wird: Compilation Error

---
transition: null
---
## Ownership

```rust {*|2-3|5-6|7}{lines:true}

let a = String::new(); // `a` owns value of type string
println!("{a}"); // Print content of `a`

let b = a; // Move ownership of string value to `b`. `a` is now invalidated, as its no longer an owner
println!("{b}"); // Print content of `b`. Same as previous println!("{a}");
println!("{a}"); // Compilation error: a was moved

```

---
transition: null
---

## Ownership

```rust {*|2-5|7|9|10}{lines:true}

fn own_and_drop(a: String) {
  println!("value of a = {a} will now get dropped");
  // Drop happens here automatically, since a is running out of scope;
}

let a = String::new();

own_and_drop(a);
println!("{a}"); // Compilation error: a was moved

```

<v-click>

- Ownership kann an Funktionen und auch Strukturen übertragen werden
- Alles was Werte von Typ `T` empfangen kann, kann Owner von einem Wert des Typen `T` werden
</v-click>

---
transition: null
---

## Ownership

```rust {*|1-4|6-7}{lines:true}
fn create_and_return() -> String {
  let a = String::from("abc");
  a // release ownership to the recipient of the return value
}

let a = create_and_return(); // a takes ownership
println!("{a}"); // Prints "abc"
```

<v-click>

- Ownership kann auch aus Funktionen heraus übertragen werden. Return gibt Ownership aus Funktionsscope nach außen frei
</v-click>

---
transition: null
---

## Ownership

```rust {*|1|2-3}{lines:true}
let mut s = String::from("abc");
s = String::from("cdf"); // "abc" is freed here, and s owns "cdf" now!
println!("{s}"); // will print "cdf"
```

<v-click>

- Ownership kann überschrieben werden, wodurch der alte "Besitz" freigegeben wird
</v-click>

---
transition: null
---

## Ownership

```rust {*|1|2|4}{lines:true}
let a = 10;
let b = a;

println!("{a}"); // Ok: ???
```

<v-click>

Warum ist das OK?
- Ownership Regel werden Augenscheinlich verletzt
</v-click>

---
transition: null
---

## Copyable Types

- Einfach zu kopierende Typen
  - Heißt: Keine Deep Copy notwending
  - Copy Interface kann aber für jeden Typen implementiert werden
- Jeder Typ, der Copy Interface implementiert, kann ganz einfach kopiert werden
  - Dadurch zwei Werte und zwei Owner. Es wird nicht gemoved   

```rust {*}{lines:true}
let a = 10;
let b = a;

println!("{a}"); // Ok: because a.copy() was called implicitly, otherwise Compiler Error
```

---
transition: null
---
## Borrowing

- Klassische Referenzen
- Aber:
  - Unterscheidung zwischen änderbaren und nicht-änderbaren Referenzen
    - Vorbereitung für Multithreading
    - Maximal eine änderbare Referenz gleichzeitig
    - Beliebig viele nicht-änderbare Referenzen, solange es keine änderbare Referenz gibt
  - Referenz darf niemals länger als der richtige Owner leben (Lifetimes)

---
transition: null
---

## Borrowing

````md magic-move
```rust {*|1|3-4|5-6|8}{lines:true}
let mut a = String::new();

let b = &a;
let c = &c;
println!("{b}");
println!("{c}");

let d = &mut a; // Compilation error: a is already borrowed
```

```rust {*|2-7|9-10}{lines:true}
let mut a = String::new();
{
  let b = &a;
  let c = &c;
  println!("{b}");
  println!("{c}");
}

let d = &mut a; // Ok
println!("{d}");
```
````

---
transition: null
---
## Borrowing

````md magic-move
```rust {*|1|2|2-3}{lines:true}
let mut a = String::new();
let b = &mut a;
let c = &mut b;
```
```rust {2-3}{lines:true}
let mut a = String::new();
let b = &mut a; // First borrow occurs here
let c = &mut b; // Compilation Error: a is already mutably borrowed
```
````

---
transition: null
---
## Borrowing

```rust {*|1-4|6|8|9}{lines:true}
fn own_and_drop(a: &String) {
  println!("value of a = {a} will now get dropped");
  // Drop does not happen, only reference is dropped, which does nothing
}

let a = String::new();

own_and_drop(&a);
println!("{a}"); // Ok
```

---
transition: null
---

## Borrowing

```rust {1-2|4|6-7}{lines:true}
let foo = String::new();
let bar = &foo;

bar.push_str("hello"); // Compilation Error: bar is not mutable

let bar = &mut foo; // Compilation Error: foo is not mutable
bar.push_str("hello"); 
```

---
transition: null
---
## Borrowing -- Lifetimes

```rust {*}{lines:true}
let foo: &String = {
  let bar = String::new();
  &bar // Compilation Error: bar does not live long enough
};
```

---
transition: null
---
## Borrowing -- Lifetimes

- Teil jedes Referenztypen
- Oft implizit
- Kann explizit mit angegeben werden:
  - `&'a T` (für Referenz des Typen `T` mit Lifetime `'a`)
  - `&'a mut T` (für änderbare Referenz des Typen `T` mit Lifetime `'a`)
- Benennt Scope, in dem die Variable lebt (sowie parent Scopes)

---
transition: null
---
## Borrowing -- Lifetimes

````md magic-move
```rust {*|1|2,3|4,5|1}{lines:true}
fn either(left: bool, a: &String, b: &String) -> &String {
  if left {
    a
  } else {
    b
  }
}
```
```rust {1}{lines:true}
fn either<'a>(left: bool, a: &'a String, b: &'a String) -> &'a String {
  if left {
    a
  } else {
    b
  }
}
```
````

<div v-click="[4, 5]">

Compilation Error?? Missing Named Lifetimes
</div>

<div v-click="6">

Ok
- Strings `a` und `b` müssen `'a` lang leben, und das Ergebnis wird auch `'a` lang leben
</div>

---
transition: null
---
## Borrowing -- Lifetimes

````md magic-move
```rust {*|1-3|2|5-6,2|8-10,2|13-14}{lines:true}
struct MyStruct {
  value: &String, 
}

let value = String::from("abc");                // -----------'a
let mut my_struct = MyStruct { value: &value }; //            |
                                                //            |
{                                               //            |
  let new_value = String::from("foo");          // -------'b  |
  my_struct.value = &new_value;                 // ------- +  |
}                                               //            |
                                                //            |
// `my_struct` is only valid for lifetime 'a    //            |
println!("{}", my_struct.value);                // ---------- +                             
```
```rust {1-3}{lines:true}
struct MyStruct {
  value: &String, // Already compilation error
}

let value = String::from("abc");                // -----------'a
let mut my_struct = MyStruct { value: &value }; //            |
                                                //            |
{                                               //            |
  let new_value = String::from("foo");          // -------'b  |
  my_struct.value = &new_value;                 // ------- +  |
}                                               //            |
                                                //            |
// `my_struct` is only valid for lifetime 'a    //            |
println!("{}", my_struct.value);                // ---------- +                             
```
```rust {1-3|9-11}{lines:true}
struct MyStruct<'a> {
  value: &'a String,
}

let value = String::from("abc");                // -----------'a
let mut my_struct = MyStruct { value: &value }; //            |
                                                //            |
{                                               //            |
  let new_value = String::from("foo");          // -------'b  |
  // Compilation Error: my_struct is of type MyStruct<'a>  |  |
  my_struct.value = &new_value;                 // ------- +  |
}                                               //            |
                                                //            |
// `my_struct` is only valid for lifetime 'b    //            |
println!("{}", my_struct.value);                // ---------- +                             
```
````


---
transition: null
---
# Schwierigkeiten

- Kompiler ist sehr Penibel
  - Viele Meldungen, die kein Bug sind, aber eventuell zu einem Memory Bug führen könnten 
- Shared State nur über Umwege möglich
  - Reference Counting (`RC`) und Änderbare Speicherzellen, die zur Laufzeit geprüft werden (`RefCell`)
- Cross Referencing: A zeigt auf B, B auf A. Auch hier wieder `RC` und `RefCell` notwendig
- Alternativ: Fallback zu echten Pointern über `unsafe`-Code
- Zur not so viel wie möglich value.clone() nutzen, falls Performance und Speichernutzung nicht all zu wichtig sind


---
transition: null
---
## Schwierigkeiten -- Shared State

````md magic-move
```rust {1-3|5-8|10-13|16-19|22-23}{lines:true}
struct State {
  counter: u32,
}

struct A<'s> {
  state: &'s State
  name: String,
}

struct B<'s> {
  state: &'s State
  value: f32,
}


let state = State {counter: 0}; // Init state

let a = A { state: &state};
let b = B { state: &state};


a.state.counter += 1; // Error: Cannot change immutable reference
b.state.counter -= 1; // Error: Cannot change immutable reference
```
```rust {5-13|17-18}{lines:true}
struct State {
  counter: u32,
}

struct A<'s> {
  state: &'s mut State
  name: String,
}

struct B<'s> {
  state: &'s mut State
  value: f32,
}

let state = State {counter: 0}; // Init state

let a = A { state: &mut state}; // First mutable borrow occurs here
let b = B { state: &mut state}; // Compilation Error: second mutable borrow
```
```rust {5-13|15|17-18}{lines:true}
struct State {
  counter: u32,
}

struct A<'s> {
  state: &'s RefCell<State>
  name: String,
}

struct B<'s> {
  state: &'s RefCell<State>
  value: f32,
}

let state = RefCell::new(State {counter: 0}); // Init state

let a = A { state: &state}; // OK, but only with lifetime
let b = B { state: &state}; // OK, but only with lifetime
```
```rust {15-21|16,20}{lines:true}
struct State {
  counter: u32,
}

struct A<'s> {
  state: &'s RefCell<State>
  name: String,
}

struct B<'s> {
  state: &'s RefCell<State>
  value: f32,
}

let (a, b) = {
  let state = RefCell::new(State {counter: 0}); // Init state

  let a = A { state: &state}; // OK, but only with lifetime
  let b = B { state: &state}; // OK, but only with lifetime
  (a, b) // Error, state does not live long enough
};
```
```rust {15-21}{lines:true}
struct State {
  counter: u32,
}

struct A {
  state: Rc<RefCell<State>>
  name: String,
}

struct B {
  state: Rc<RefCell<State>>
  value: f32,
}

let (a, b) = {
  let state = Rc::new(RefCell::new(State {counter: 0})); // Init state

  let a = A { state: Rc::clone(&state)}; // OK
  let b = B { state: Rc::clone(&state)}; // OK
  (a, b) // Ok
};
```
````

---
transition: null
---
## Schwierigkeiten -- Impossible Lifetimes

```rust {1-4|6|8-11}{lines:true}
struct A {
  value: String,
  reference: &String,
}

let string = String::from("abc");

let a = A {
  value: string,
  reference: &value,
};
```

<v-click>

- Grundsätzlich gültig, da nur eine nicht änderbare Referenz und ein Owner
- Aber unmöglich die Lifetime zu bestimmen
  - So lange wie `a` lebt? Aber wie lange lebt `a`? Wie komme ich an die Lifetime von `a` aus Entwicklersicht
</v-click>


---
transition: null
---
## Ausblick

- Erfahrungsgemäß werden viele Bugs bereits zur Compile Zeit ausgeschlossen
  - Fearless Concurrency (Multithreading geprüft durch den Compiler)
  - Once it Compiles, it runs (while beeing "blazingly fast and memory-efficient" - Rust Lang) (Logik-Fehler ausgeschlossen)
- Ownership und Borrow Checking eliminieren viele Fehlerklassen
  - Dazu kommt noch extrem starkes Typ-System
- In der Industrie angekommen
  - Linus Kernel
  - Microsoft Windows Kernel