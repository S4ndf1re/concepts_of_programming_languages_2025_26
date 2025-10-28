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
let a = 10; // Int
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
fn my_func(players: {PositionComponent, VelocityComponent | PlayerComponent, !BotComponent })


// 2. Alternative (Favorit)
fn my_func(players: query query (Component1, Component2) 
            where (Component3 | Component4) & Component5 & !Component6)


// 4. Alternative
fn my_func(players: <(Component1, Component2) $ (Component3 || Component4) && !Component5>)


fn my_system(
    param: query (Component1, Component2) 
            where (Component3 | Component4) & Component5 & !Component6
) {
    ...
    ...
    ...

}
```

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


## Fragen an Gips
- Wo ist die Grenze zwischen DSL, vollständiger Programmiersprache und netter Kommandozeile?
- Sollen die Scripte auch abgelegt werden (als Datei), und dann bei Programmstart, oder sogar während des Programmes ausgeführt/ interpretiert werden, oder ist nur ein REPL notwending?
- Sollen Components erzeugt werden können?
- Wie funktionieren Events in dem Dungeon?
- Gibt es Resourcen?
- Wie sieht das Level Management aus? (Beispiel BridgeGuardRiddleLevel.java)? Müssen level hintereinander gespawned werden können? Wie könnten Tasks generell in Level Eingebaut werden (das schein sehr aufwendig zu sein!)?
- Wie werden Tasks überhaupt getriggert? InteractionComponent wird irgendwie nie benutzt