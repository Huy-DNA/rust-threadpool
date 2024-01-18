# Rust Threadpool
A small Rust library for creating a thread pool. This is my implementation by memory after reading an implementation in the [The Rust Programming Language](https://doc.rust-lang.org/stable/book/) book.

The main thing that I learnt is unsurprisingly, how to implement a threadpool (although I already knew how, so may be I can say this reinforce my understanding of it).
One more important thing I also learnt is to used Github Action to perform some basic CI/CD. This is my first hand experience though:
* Automatically test the rust code.
* Automatically generate documentation in the `doc` branch.
* Deploy the rust doc automatically, although I couldn't do it with Github Action yet. The main difficulty with deploying was that it expects an `index.html` at top-level, so I had to craft up a script to generate a dummy `index.html` that automatically redirects to the real doc page.

The doc is deployed here: https://huydna.github.io/rust-threadpool
