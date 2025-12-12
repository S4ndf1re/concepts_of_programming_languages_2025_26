---
# try also 'default' to start simple
theme: academic 
# random image from a curated Unsplash collection by Anthony
# like them? see https://unsplash.com/collections/94734566/slidev
background: https://cover.sli.dev
# some information about your slides (markdown enabled)
title: Just in Time Compilation
info: |
  Basic introduction into Just in Time compilation
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
duration: 60min
hideInToc: true
layout: intro
---


# Just in Time Compilation (JIT)
Wenn interpretation zu langsam ist

16.12.2025 | Jan Kleinmann, Marius Braun




---
layout: default
hideInToc: true
---
# Agenda
<Toc maxDepth="1"/>


---
layout: two-cols
---
# Interpretieren am Beispiel von Python

```python {all|5|all} {lines:true}
bound = 100000000
print(bound)
sum = 0
for i in range(bound):
    sum += i;
```

::right::
- Execution time (python3):
  - 0 -> 1.000.000: 99,52 ms
  - 0 -> 100.000.000: 4,77 s

- Execution time (pypy --jit off):
  - 0 -> 1.000.000: 149,49 ms
  - 0 -> 100.000.000: 5,37 s

- Execution time (pypy --jit on):
  - 0 -> 1.000.000: 99,52 ms
  - 0 -> 100.000.000: 4,77 s


---
layout: figure
figureCaption: "abc"
figureUrl: img/interpreter_vs_compiler.png
figureFootnoteNumber: 1
---
## Test

<Footnotes>
  <Footnote number="1">
    google.com
  </Footnote>
</Footnotes>