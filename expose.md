# ECSInject: Injection is the key for ECS development

## Was?
- Basierend auf einfachem ECS (muss nicht Spiel sein)
- ECS haben Entitäten, Komponenten und Systeme
- Systeme über gelungene Dependency Injection definieren! (Ziel)


## Warum Spannend?
- ECS sehr beliebt in Game Engines (Dungeon, Bevy, andere)
- Nachteil bei ECS in herkömmlichen Programmiersprachen: Schwerer Zugriff zu Entitäten, die Bestimmte Komponenten enthalten. Kaum Query möglichkeiten (Pattern matching)
- Bevy löst das begrenzt, indem Rust's Typsystem verwendet wird, um zur Compile Time Queries zu erzeugen, die die Engine auflösen kann.
  - Vorteile:
    1. Einfaches abfragen von Entitäten
    2. Ausführgraphen erzeugen, um Parallelität zu erhöhen, indem Systeme anhand ihrere Dependencies gruppiert werden, wordurch viele Systeme parallel laufen können (alle die keine gleichen Bedingungen haben)
  - Nachteil:
    1. Rust hat zwar starkes typsystem, aber nicht optimiert für Dependency Injection (DI)
    2. Queries können nur Top level, und nicht über Hierarchien (Parent, Children) verarbeitet werden.
    3. Es gibt viele ECS, und as far as i know supported nur Bevy eine Systemspezifische DI Engine


## Das Wie?
- Starke Typisierung
- Pattern Matching als First Class Citizen
- Entities, Komponenten und Systeme (Funktionen) als first class citizens


## Mit der Bewertung steht und fällt alles
- Erzeugen von selben Usecases in verschiedenen ECS
- Subjektiver codevergleich
- LOC Metriken
- Lesbarkeit von Dritten beurteilen lassen
