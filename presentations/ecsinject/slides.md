---
# try also 'default' to start simple
theme: academic
# random image from a curated Unsplash collection by Anthony
# like them? see https://unsplash.com/collections/94734566/slidev
background: https://cover.sli.dev
# some information about your slides (markdown enabled)
title: ECSInject
info: |
  ## ECSInject
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

# ECSInject

First-Class-Citizen Entities, Components and Systemes


---
transition: null
hideInToc: true
---

# Agenda

<Toc maxDepth="1"/>

---
transition: null
layout: image-right
image: /img/disclaimer.jpg
---

## Disclaimer

- Prototyp
- Nicht 100\% identisch zu dem Konzept



---
transition: null
title: Konzept (Rückblick)
---

## Ein kleiner Rückblick

- Programmiersprache mit ECS als First-Class-Citizen
- Entity Component System (ECS)
  - Entitäten
  - Komponenten 
  - Systems


---
transition: null
layout: figure
figureCaption: "ECS in Unity"
figureFootnoteNumber: 1
figureUrl: "/img/ecs.png"
---

## Ein kleiner Rückblick

<Footnotes>
  <Footnote number="1">
    https://docs.unity3d.com/Packages/com.unity.entities@6.5/manual/concepts-ecs.html
  </Footnote>
</Footnotes>

---
transition: null
---

## Ein kleiner Rückblick

- ECS wird um Sprache gebaut => Sprache wird um ECS entwickelt
- Kernproblem:
  - Wie kann die Sprache in ECS integriert werden?
  - Dabei soll die Sprache als Script verstanden werden
    - Spieleentwicklung lebt von schneller iteration


---
transition: null
---

## Ein kleiner Rückblick

Bevy Game Engine
```rust
fn update_people(mut query: Query<&mut Name, With<Person>>) {
  for mut name in &mut query {
      if name.0 == "Elaina Proctor" {
          name.0 = "Elaina Hume".to_string();
          break; // We don't need to change any other names.
      }
  }
}
```

- Funktioniert gut, aber:
  - Langsame Compile-Zeiten:
    - Rust selbst braucht lange
    - Starke Macro Nutzung mit viel Code generierung
  - Einschränkungen bezüglich  Borrowing
  - Nur Eingeschränkte Query Möglichkeiten
    - Query ausdrücke werden schnell sehr komplex, da ausschließlich über Generics möglich

<Footnotes>
  <Footnote number="1">
    bevy.org
  </Footnote>
</Footnotes>


---
transition: null
title: Grammatik
layout: image-right
image: /img/grammar_dict.jpg
---

## Grammatik 

---
transition: null
---

## Grammatik 

- Lalrpop (LR(1))
  - Generiert Rust
  - Eigene Datentypen verwendbar

```rust
/// Start point of the grammar
pub Programm: Vec<AstNode> = {
    <imports:Import*> <statements:Statement*> => imports.into_iter().chain(statements).collect::<Vec<AstNode>>()
};

/// Any import, i.e. native ffi import and normal
Import: AstNode = {
    <l:@L> import <module:id> <alias:AliasRule?> semicolon <r:@R> => AstNode::new(l..r, AstNodeType::Import(module, alias)),
};

/// Any statement, meaning if, while, for, anything non returnable like function definitions, struct definitions and assignments. Additionally Returnable statements with semicolon
Statement: AstNode = {
    #[precedence(level="0")]
    NonReturnable,
    If, //Note: das if ist hier nicht zweideutig, wegen dem LR(1)-Lookup von dem Semikolon
    While,
    For,
    #[precedence(level="1")] #[assoc(side="left")]
    ReturnableStatement,
};

```

---
transition: null
---

## Grammatik -- Systeme

````md magic-move
```rust
system s1 (a: <P1>, b: <P2>, c: <P3>)
querying
  <P1> as List with {Entity, Component1, Component2 % !Component3}
  <P2> as List with {Entity, Component1, 
% #Parent: {Component2, Component3}, Any<#Children>{Component4}, Component6 && Component7 && !Component8 || Component9}
  <P3> as Single with {Entity, Camera % MainCamera} {
}
```
```rust
system s1 (a: <P1>, b: <P2>, c: <P3>)
querying
  <P1> as List with {Entity, Component1, Component2 % {!Component3}},
  <P2> as List with {Entity, Component1, % {Component6 && Component7 && !Component8 || Component9}},
  <P3> as Single with {Entity, Camera % {MainCamera}} {
}
```
````

<v-click>

- Keine Parent/Any-Children Bedingung mehr
- Extra geschweifte Klammern um Bedingung
- Komma hinter Parameter-Bedingungen 
</v-click>


---
transition: null
---

## Grammatik -- Systeme

````md magic-move
```rust
register s1; 
register s1 -> s2 -> s3 -> s4;
register s1 after s4;
register s1 before s4;


group PreUpdate {
  s1, s2, s3,
},

group Update {
  s4,
  s5 -> s6,
  s6 -> s7,
  s6 -> s8,
}

register group PreUpdate -> Update;
```
```rust
register s1; 
register s1 -> s2 -> s3 -> s4;
```
````

<v-click>

- Gruppen nicht mehr von Interpreter Unterstützt
  - Grammatik weiterhin, aber ECS Prototyp unterstützt das nicht
- Keine Sortierung der Systeme mehr
</v-click>

---
transition: null
---

## Grammatik -- Entity Management

```rust
create entity player
  with
    Position2d {x: 10, y: 10},
    Velocity {x: 3, y: 3},
    MarkerComponent {};

player += Player {lives: 5};
player -= MarkerComponent;

despawn player;
```

---
transition: null
---

## Grammatik -- Sonstige Änderungen

- Null / None Value: 
  - Original über Rustähnliche Result und Option Typen
  - Jetzt: Result / Option als Builtin Structs definiert (nicht mehr Teil der Syntax)
- Listen: 
  - Listen nur noch wie Arrays == statisch
  - Dynamische Listen auch nur über Builtin Typen abgebildet


---
transition: null
layout: two-cols-header
---

## Grammatik -- Abstract Syntax Tree

::left::

- 25 Varianten
  - For, ForEach, While, If
  - TypeDef, Assignment, Declaration
  - Weak
  - Register
  - ...

::right::

```rust
Import(Module, Option<Alias>),
Int(i64),
Float(f64),
String(String),
Bool(bool),
List(Vec<Box<AstNode>>),
Declaration {
    new_symbol: Symbol,
    expression: Box<AstNode>,
    assumed_type: Option<TypeSymbol>,
},
AssignmentOp {
    recipient: Vec<MemberAccess>,
    operation: AssignmentOperations,
    expression: Box<AstNode>,
},
[...]
InfixCall(Box<AstNode>, InfixOperator, Box<AstNode>),
PrefixCall(PrefixOperator, Box<AstNode>),
MemberCall {
    calls: Vec<MemberAccess>,
},
[...]
```

---
transition: null
---

## Grammatik -- Abstract Syntax Tree

```rust
struct A {
    a: float,
    fn my_function_name(a: int, b: string, c: MyStruct): float {
    }
    c: String,
}
```

<img src="/img/struct.png" class="m-auto p-1 w-2/5"/>


---
transition: null
title: Entity Component System
layout: image-right
image: /img/rails.jpg
---

## Entity Component System


---
transition: null
---

## Entity Component System

- Möglichst einfach, wenn möglich Third-Party
- Aber:
  - Muss sowohl in Rust (wegen Interpreter) als auch der Interpretierten Sprache gut itegriert sein
  
<v-click>

- Hieß für mich: Eigenes ECS
  - Aber stark an Bevys ECS angelehnt
  - Extrem einfach gehalten 
</v-click>

---
transition: null
---

## Entity Component System

```rust{*|1-7|8|10}
/// raylib_input_system handle simple raylib input management
/// Using WasdControl and RaylibHandle (as in SingleQuery<RaylibHandle>)
fn raylib_input_system(world: &World, raylib_handler: SingleQuery<RaylibHandle>) {
    if let Err(err) = raylib_input_helper(world, raylib_handler) {
        println!("{err}");
    }
}

world.add_system(raylib_input_system);
world.run();
```
```rust{*|1-2|4-10|5-9}
let mut entity = world.spawn();
entity.add_component(RaylibHandle(rl, thread));

let mut entity = world.spawn();
let (instance, _component) = instantiate_component_as_t!(scope, "WasdControl" => WasdControl,
        vec![("w".to_owned(), Box::new(InterpreterValue::Bool(false))),
            ("a".to_owned(), Box::new(InterpreterValue::Bool(false))),
            ("s".to_owned(), Box::new(InterpreterValue::Bool(false))),
            ("d".to_owned(), Box::new(InterpreterValue::Bool(false)))].into_iter().collect::<HashMap<_, _>>());
entity.add_component(instance);
```


---
transition: null
title: Interpreter
layout: image-right
image: /img/calculator.jpg
---

## Interpreter


---
transition: null
---

## Interpreter

- Dynamisch Typisiert
- Leichtes Preprocessing
- Aufgeteilt in: Parsing (Grammatik) -> Preprocessing -> Interpretation
- Memory Model: Reference Counting für Objekte (Strukturen und Komponenten)
- Recursive Descent Interpretation


---
transition: null
layout: two-cols-header
---

## Interpreter -- Preprocessing

::left::
- Filtert
  - Funktions- / System-Definitionen 
  - Strukturen- / Componenten-Definitionen
  - Imports
  - System Registrierungen
  - Identifiziert u.a. auch `main`-Funktion
- Registriert Builtin-Strukturen und Komponenten
  - BuiltinList
  - Optional
  - Result
  - WorldObj
- Prüft Existenz von angegebenen Typen
  - Auch später definierte Typen werden gefunden


::right::
```rust
AstNodeType::TypeDef {
    typename,
    typedef,
    execution_body,
} => {
  match typedef {
    AstTypeDefinition::Component(attributes) => {
        let struct_def =
            TypeSymbol::strong(
              TypeSymbolType::Component(ComponentType {
                name: typename.clone(),
                fields: attributes,
            }));
        scope
            .borrow_mut()
            .declare_type(typename, struct_def, 
                     true, node.range.clone())
            .map_err(|err| ErrorWithRange {
                err,
                range: node.range.clone(),
            })?;
    },
  }
}

```

---
transition: null
layout: two-cols-header
---

## Interpreter -- Types

::left::

- Jeder Interpreter Value hat definierten Typen (Type Symbol + Type Symbol Type)
- Gibt an ob strong oder weak Reference
- Häufig besonders bei Builtins: `TypeSymbolType::Any`
- Funktions- und Systemtypen enthalten zusätzlich den ausführbaren AST, oder Builtin-Callback 


::right::

```rust {*}{lines:true}
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum TypeSymbolType {
    Int,
    Float,
    Bool,
    String,
    Symbol(Symbol),
    List(Box<TypeSymbol>),
    Struct(StructType),
    Component(ComponentType),
    Function(FunctionType),
    System(SystemType),
    SelfType,
    Any,
    Entity,
}
```

---
transition: null
---

## Interpreter -- Types

- Besonderheit bei Structs und Komponenten:
  - Vorgefertigte Objektinstanzen, die Klone von sich erzeugen können
  - Dadurch können Builtins und Interpretierte Structs die selbe Type Definition teilen


````md magic-move
```rust {*|8}{lines:true}
#[derive(Debug, Clone)]
pub struct StructType {
  pub name: Symbol,
  pub fields: Vec<(Symbol, TypeSymbol)>,
  // Methods are assumed to start with "self"
  pub methods: Vec<(Symbol, FunctionType)>,
  pub statics: Vec<(Symbol, FunctionType)>,
  pub prefab: Option<Rc<dyn BuiltinStruct>>,
}
```
```rust {*|1|7-9|12-15}{lines:true}
impl Instantiable for StructType {
  fn instantiate(
      &self,
      scope: Rc<RefCell<Scope>>,
      params: HashMap<Symbol, Box<InterpreterValue>>,
  ) -> Result<InterpreterValue, Error> {
      if let Some(prefab) = &self.prefab {
          return prefab.instantiate(scope, params);
      }
      [...error handling...]

      let struct_value =
          InterpreterValue::Struct(self.name.clone(), scope, params).make_reference_counted()?;

      Ok(struct_value)
  }
}
```
```rust {*|2,3}{lines:true}
pub trait BuiltinStruct: Debug + ScopeLike + Instantiable {
  fn to_type(self) -> Result<TypeSymbol, Error>;
  fn resolve_builtin_type(&self) -> Option<TypeSymbol>;
  fn name(&self) -> String;
}
```
```rust {*|1|4-5|8,10,12,14,16}{lines:true}
#[derive(Debug, BuiltinStruct)]
pub struct BuiltinList {
    pub container: Vec<InterpreterValue>,
    #[scope]
    pub scope: Rc<RefCell<Scope>>,
}

#[expose_funcs]
impl BuiltinList {
  #[expose]
  pub fn get(&mut self, idx: InterpreterValue) -> Result<InterpreterValue, Error> { [...] }
  #[expose]
  pub fn set(&mut self, idx: InterpreterValue, value: InterpreterValue) -> Result<InterpreterValue, Error> { [...] }
  #[expose]
  pub fn push(&mut self, value: InterpreterValue) -> Result<InterpreterValue, Error> { [...] }
  #[expose]
  pub fn pop(&mut self) -> Result<InterpreterValue, Error> { [...] }
}
```
````


---
transition: null
layout: two-cols-header
---

## Interpreter -- Values

::left::

- InterpreterValue ist Algebraischer Datentyp für Werte
- Reference Counting (+ Weak References)
- Enhält auch Integration mit ECS
- Importierte Module
- Funktionen und Methoden
- Strukturen (+ Builtin Strukturen)

::right::

```rust {*|3-6|9-11|13-22}{lines:true}
#[derive(Clone, Debug)]
pub enum InterpreterValue {
  Int(i64),
  Float(f64),
  String(String),
  Bool(bool),
  List(Vec<InterpreterValue>),
  [...]
  // Reference counted values (everything afaik)
  Weak(Weak<RefCell<InterpreterValue>>),
  Strong(Rc<RefCell<InterpreterValue>>),
  [...]
  // ECS Intergration
  Entity(EntityIndex),
  Component(
      Symbol,
      Rc<RefCell<Scope>>,
      HashMap<Symbol, Box<InterpreterValue>>,
  ),
  BuiltinComponent(Symbol, Rc<RefCell<dyn BuiltinComponent>>),
  // System execution body is contained in its type definition,
  System(Symbol), 
  [...]
}
```

---
transition: null
---

## Interpreter -- Eval

- `eval_node` Recursive Descent Interpretation

````md magic-move
```rust {*}{lines: true}
pub fn eval_node(&mut self, node: &AstNode, world: &World, file: &'static str,
) -> Result<IsReturn, ErrorWithRange> {
  let evaluated = match &node.type_of {
      // Primitives
      AstNodeType::Bool(b) => IsReturn::NoReturn(InterpreterValue::Bool(*b)),
      AstNodeType::Int(i) => IsReturn::NoReturn(InterpreterValue::Int(*i)),
      AstNodeType::Float(f) => IsReturn::NoReturn(InterpreterValue::Float(*f)),
      AstNodeType::String(s) => IsReturn::NoReturn(InterpreterValue::String(s.clone())),
      AstNodeType::List(values) => IsReturn::NoReturn(self.eval_list(values, world, file)?),
      AstNodeType::Weak(inner) => IsReturn::NoReturn(self.eval_weak(inner.as_ref(), world, file)?),
      AstNodeType::InfixCall(left, op, right) => IsReturn::NoReturn(self.eval_infix_call(
          left.as_ref(),
          op,
          right.as_ref(),
          world,
          file,
      )?),
    [...]
  }
}
```
```rust {1-4|6-13}{lines: true}
scoped!(self, {
    let res = self.eval_nodes(body, world, file)?;
    return_on_return!(res);
});

macro_rules! scoped {
    ($s:ident, $inner:block) => {{
        $s.push_scope();
        let ret = { $inner };
        $s.pop_scope();
        ret
    }};
}
```
````



---
transition: null
title: Error Handling
layout: image-right
image: /img/alarm.jpg
---

## Error Handling


---
transition: null
---

## Error Handling

- Immens hilfreich bei der Fehlersuche
- Immer wenn möglich Quelltext mitsamt Zeilen-/Tokenangaben referenzieren
- Bestenfalls farbliche Fehlerausgaben
- [Annotate Snipptes](https://docs.rs/annotate-snippets/latest/annotate_snippets/)


<img v-click="['1', '2']" src="/img/error_1.png" class="absolute transform -translate-x-1/2 -translate-y-1/2 top-2/3 left-1/2 w-2/3 m-auto"/>
<img v-click="['2', '3']" src="/img/error_2.png" class="absolute transform -translate-x-1/2 -translate-y-1/2 top-2/3 left-1/2 w-2/3 m-auto"/>
<img v-click="3" src="/img/error_3.png" class="absolute transform -translate-x-1/2 -translate-y-1/2 top-2/3 left-1/2 w-2/3 m-auto"/>

---
transition: null
title: ECS Integration 
layout: image-right
image: /img/integration.jpg
---

## ECS Integration

---
transition: null
---

## ECS Integration

````md magic-move
```rust {*|1-4|6-11|13|15-20}{lines: true}
component Velocity {
    x: int,
    y: int,
}

system apply_input_left(left_bar: L, input: I)
    querying L as List with { Position2d, RectangleShape % { LeftMarker && BarMarker } },
             I as Single with { WasdControl }
{
  [...]
}

register apply_input_left;

create entity left_bar
    with
        LeftMarker{},
        BarMarker{},
        Position2d{ x: 40.0, y: 320.0 },
        RectangleShape{ w: 20.0, h: 100.0 };
```
```rust {15-19}{lines: true}
component Velocity {
    x: int,
    y: int,
}

system apply_input_left(left_bar: L, input: I)
    querying L as List with { Position2d, RectangleShape % { LeftMarker && BarMarker } },
             I as Single with { WasdControl }
{
  [...]
}

register apply_input_left;

create entity left_bar;
left_bar += LeftMarker{};
left_bar += BarMarker{};
left_bar += Position2d{ x: 40.0, y: 320.0};
left_bar += RectangleShape{ w: 20.0, h: 100.0 };
```
```rust {15-20}{lines: true}
component Velocity {
    x: int,
    y: int,
}

system apply_input_left(left_bar: L, input: I)
    querying L as List with { Position2d, RectangleShape % { LeftMarker && BarMarker } },
             I as Single with { WasdControl }
{
  [...]
}

register apply_input_left;

create entity left_bar;
left_bar += LeftMarker{};
left_bar += BarMarker{};
left_bar += Position2d{ x: 40.0, y: 320.0};
left_bar += RectangleShape{ w: 20.0, h: 100.0 };
left_bar -= LeftMarker;
```
````

---
transition: null
---

## ECS Integration

```rust {*|1-3|6-8}{lines: true}
system do_something(obj: P, world: W)
  querying P as List with { Entity, AnyComponent },
           W as World
{
  for(o in obj) {
    if (o.x > 100) {
      world.stop();
    }
    o.x += 1;
    println(o.x);
  }
}
```

---
transition: null
title: Tests
layout: image-right
image: /img/tests.jpg
---

## Tests


---
transition: null
layout: image-right
image: /img/unit_tests_2.png
---

## Tests

- Unit tests
  - Parser
  - Interpreter
- Auch hier wieder: Fehler Ausgabe extrem hilfreich

---
transition: null
---

## Tests

- RayLib integration
  - Builtin Komponenten
    - 2d Position
    - 2d Rechtecke 
    - Wasd Input
  - Builtin Systeme
    - User Input => Füllt Wasd komponente
    - Raylib Render => Malt Rechtecke anhand der Größe und Position
  - Builtin Funktion
    - `raylib_init` => initialisiert Raylib und öffnet fenster


---
transition: null
title: Ausblick
layout: image-right
image: /img/ausblick.jpg
---

## Ausblick


---
transition: null
---

## Ausblick

- var\[0\]\[0\] => Fehler in der Grammatik, einfache Änderung nicht möglich, da ambiguity
- Fehlendes Statisches Typechecking für nahezu alle Operationen
  - Häufig als TODO
  - Es wird lediglich geprüft, ob alle explizit benannten Typen existieren
- Interpreter value wir häufig kopiert (Performance), da nur Objekte als Reference Counted Objekte verwaltet werden

---
transition: null
---

## Lessons Learned 

- `InterpreterValue` sehr fragil
  - Weniger Kopieren, mehr Referenzieren (auch RC)
- Funktionstypen (`TypeSymbol`) sollten nicht den Funktionsbody enthalten
  - Lambdas / Annonyme Funktionen
  - Funktionsbody lieber in `InterpreterValue` hinterlegen
- LR(1) ist nett
  - Viel ambiguity
  - Schwer, ambiguity schön zu entfernen (var.\[\].\[\])
- Besseres Typechecking erleichtert Leben
- Insgesamt fehlen Typen: Map, Tuple, sowie das dazugehörige unpacking 
- Builtin Library aufbauen dauert lange
