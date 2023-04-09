# turir

turing machine based programming language.

[examples](./examples/)

# Syntax

```rust
#halt <HALT-STATE-SYMBOL> <OTHER-HALT-STATE-SYMBOL> // multiple are accepted
#run <INITIAL-TAPE> <INITAL-STATE>
#run <INITIAL-TAPE> <INITAL-STATE> // multiple runs are accepted

<CURRENT-STATE> <READ-SYMBOL> <WRITE-SYMBOL> <TAPE-DIRECTION> <NEW-STATE>
```
basically a turing machine..

Binary increment example:
```rust
#halt H // halt state

#run [0 0 0 0 1] I // this produces -> [1 0 0 0 1]
#run [1 1 1 1 0] I // this produces -> [0 0 0 0 1]
#run [1 1 0 1 1] I // this produces -> [0 0 1 1 1]

I 0 1 -> H
I 1 0 -> I
```
