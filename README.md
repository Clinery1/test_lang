# About
This is my first ***functioning*** programming language I am posting publicly. It is going to be
used for some experiments. I also plan to use it for daily use if things work out and I get
compilation working.

For now, the language is dynamically typed. In the future I plan to add a mostly-structural linear
type system and probably Rust traits (I think they are a great idea).

We have pretty printed errors already, just like Rust, but they have no color. That is planned
though.


# What is different about this language?
- We require parenthesis around nested binary and unary operations.
- Variables, by default, are moved when used in an expression. The `copy` expression is used to get
    around this.
- We have 2 forms of mutability: Reassign and Mutate. See below for an explanation.
- There is a print statement. This is not unheard of, but it is uncommon. I will likely remove it
    later, but it is useful until I put the effort to add a standard/core library.


# Syntax
The syntax is similar to JavaScript, but also borrows from Rust and Python.

We do use newlines OR semicolons. The only time the parser ignores a newline is when the next line
is a field access/method call.

I need to write a formal EBNF grammar, but here are a few examples for now:
```javascript
function sayHello(name) {
    print "Hello there "
    print name
    print "!\n"
}
```
```javascript
function fib(n) {
    if (copy n) <= 0 {
        return 0
    } else if (copy n) == 1 {
        return 1
    } else {
        return fib((copy n) - 1) + fib(n - 2)
    }
}
```
```javascript
function fizzBuzz(n) {
    let five = ((copy n) % 5) == 0
    let three = ((copy n) % 3) == 0

    if (copy five) and (copy three) {
        print "FizzBuzz\n"
    } else if three {
        print "Fizz\n"
    } else if five {
        print "Buzz\n"
    } else {
        print n
        print "\n"
    }
}
```


# Mutability rules
There are 3 keyword that effect mutability of variables: `let`, `var`, and `mut`.

Either `let` or `var` are required to create a variable, but `mut` can be added before either of
them.

## Reassign
Reassign privilege is given with the `var` keyword. It allows you to use the `set` statement on a
variable when it already has a value.

## Mutate
Mutate privilege is given with the `mut` modifier keyword. It allows you to mutate the data in the
variable through references, but NOT reassign data to the reference/variable.


# Plans for the language
- Classes (just needs implementation)
- Anonymous objects (needs parsing and implementation)
- A REPL
- Multi-file support
- Colored errors!
- Static typing: semi-structural and semi-linear
- Static analysis (comes with the static typing)
- Compilation to WASM, via cranelift, or to C
- Macros?
