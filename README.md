# Pirogue

Pirogue is a concatenative language built to explore concatenative combinators and other ideas outlined [here](https://hypercubed.github.io/joy/joy.html). It's inspired by [Joy](https://wiki.c2.com/?JoyLanguage), [Forth](https://wiki.c2.com/?ForthLanguage), and [Stck](https://github.com/teodoran/stck).

## Name

<img align="right" width="100" src="pirogue.jpg">

A Cajun Pirogue is a small, flat bottomed boat built for the shallow marshes and bayous in Louisiana. It's said that a pirogue can "can float on a heavy dew".

## Running

With rust installed:

```
cargo run --release
```

## Syntax

The syntax is just a flat series of tokens. There are only 3 datatypes: ints, symbols, and quotes.


For example to add two numbers, push both onto the stack then use the builtin `+` symbol to add them.

```
>> 1 2 +
[3]
```

If a symbol is not defined, it will just be pushed onto the top of the stack like any other value:

```
>> 1 a 2 b
[1 a 2 b]
```

A quote is any valid program and can be pushed onto the stack. You enclose in `[` `]` to denote a quote:

```
>> [1 2 +]
[[1 2 +]]
```

You can think of a quote like an anonymous function. It can be "evaluated" with `i`:

```
>> [1 2 +] i
[3]
```

With this, you can define new functions with the `define` function. It expects a quote (the function body) and a symbol (the function name) on top of the stack:

```
>> [1 +] add1 define
[]
>> 41 add1
[42]
```

## Combinators and Stack Manipulation

With these primitives you can do a lot, but in order to create higher level functions and syntax we provide some combinators and stack manipulation functions:

`swap`, `zap`, and `dup`, and `over` are all familiar to languages like forth:

```
>> a b
[a b]
>> swap
[b a]
>> swap
[a b]
>> dup
[a b b]
>> zap
[a b]
>> over
[a b a]
```

There are some functions that operate on the top 3 elements, like the classic `rot`:

```
>> a b c
[a b c]
>> rot
[b c a]
>> rot
[c a b]
>> rot
[a b c]
```

There are also some functions that operate on quotes allowing you to write programs that
manipulate other programs. You can learn more about these in this article about [concatenative combinators in Joy](http://tunes.org/~iepos/joy.html).

`dip` expects two quotes on the stack, it evals the top and swaps the order: 

```
>> [b] [a]
[[b] [a]]
>> dip
[a [b]]
```

`unit` is sort of the inverse of `i` in that it wraps the quote in a quote:

```
>> [a]
[[a]]
>> unit
[[[a]]]
>> unit
[[[[a]]]]
>> i
[[[a]]]
>> i
[[a]]
```

`cat` allows you to concatentate two quotes:

```
>> [b] [a] cat
[[b a]]
```

`cons` can do the same but leaves the bottom element quoted:

```
>> [b] [a] cons
[[[b] a]]
```

