## Typisierung
- Einfache Typen
- Int, Float, String, Bool, Array(List), (Map)?, Struct (einfaches Gruppieren von Attributen), Function Types, Vielleicht Tupel

## Kommentare
```c
// Dies ist ein kommentar, Mehrzeilige Kommentare gibt es nicht
```

## Variablen
```rust
let a: Int = 10;
// automatische typzuweisung
let b = 10;
```

## Typen
```rust
let a: Int = 10; // Int
let b: Float = 10.1; // Float
let c: String = "Hello"; // String
let h: Bool = false;
let d: Int[] = Int[];  // Array(list)
let e: Int->String = Int->String; // Map // TODO // Vielleicht als funktion nutzbar

struct TestStruct {
    a: Int,
    b: Float,
    c: String,
    d: TestStruct
}

let f: TestStruct = TestStruct rest default;

let g = TestStruct with {
    a : 123, 
    b: 1231.0
} rest default;


fn ab(entityStream: Entites) {
}



fn asdfasdfasf (adsfalsfkj){
    adsflajsdfoasdf;
    asdflasjf;
    adsf;
}

register abc
register <system> on <event>

```
## Queries
```rust
fn abc(query: ((&PositionComponent, &VelocityComponent, Entity), Without<PlayerComponent>)) {
    if query.is_empty() return


    for entry in query {
        entry.0 += entry.1

        if entry.0 > (100, 100) {
            add OutOfBoundsComponent to entry.2
            entry.1 = -entry.1
        } 

        if entry.0 < (100, 100) {
            remove OutOfBoundsComponent from entry.2
        }
    }
}
```

## Keywords
register <>

## Null / nil / None / Option / Bools

## Structs

## Maps

## Funktionsdefinition

```c
void main() {

}

```

## Memory Management


## Concept Queries
```rust

// 1. Alternative
// Wird dann automatisch dependency injected
let tuple: {(Component1, Component2) | Component3, !Component4};

fn my_func(players: {PositionComponent, VelocityComponent | PlayerComponent, !BotComponent })


// 2. Alternative
let tuple: query (Component1, Component2) 
            with (Component3, Component4)
            without Component6;

fn my_func(players: query (Entity, PositionComponent, VelocityComponent) with PlayerComponent without BotComponent)

// 3. Alternative
let tuple: Query<(Component1, Component2), (Component3, Component4), (Component5)>;

fn my_func(players: Query<(Component1, Component2), (Component3, Component4), (Component5)>)


// 3. Alternative
let tuple: <(Component1, Component2) $ (Component3 || Component4) && !Component5>;

fn my_func(players: <(Component1, Component2) $ (Component3 || Component4) && !Component5>)
```


## Fragen an Gips
- Wo ist die Grenze zwischen DSL, vollständiger Programmiersprache und netter Kommandozeile?
- Sollen die Scripte auch abgelegt werden (als Datei), und dann bei Programmstart, oder sogar während des Programmes ausgeführt/ interpretiert werden, oder ist nur ein REPL notwending?
