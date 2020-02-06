Summary: Async and network programming in Rust
Id: async-network-dev-workshop
Categories: rust
Status: Published
Feedback Link: https://github.com/avranju/rust-async-nw-workshop/issues
Authors: Rajasekharan Vengalil

# Async and network programming in Rust

## What is a Future?
Duration: 1

A [Future](https://doc.rust-lang.org/std/future/trait.Future.html) is a trait
that represents a type that will yield a value at some point in the future. It
is analogus to JavaScript
[promises](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise)
and .NET
[tasks](https://docs.microsoft.com/en-us/dotnet/api/system.threading.tasks.task-1?view=netcore-3.1).
Here's how the `Future` trait has been defined in the standard library:

```rust
pub trait Future {
    type Output;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>;
}
```

To get a sense of what implementing a `Future` looks like, here is a `Future`
that immediately yields the value provided to it (similar to .NET's
`Task.FromResult` or JavaScript's  `Promise.resolve`).

```rust
use std::pin::Pin;
use std::future::Future;
use std::task::{Poll, Context};

struct Ready<T>(Option<T>);

impl<T: Unpin> Future for Ready<T> {
    type Output = T;
    
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(self.0.take().unwrap())
    }
}
```

Positive
: To see this in action take a look at this [sample playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=bf982a52a93559b4b892ec2cb589c54d).

Positive
: We'll talk about pinning and the `Unpin` trait that you see in the code snippet above later in the workshop.

## Let's hand-write a Future

So let's implement the world's worst delay timer future.

## Let's write an executor

This might seem like a tall order, but trust me, its not so bad.

## Feedback

It'd be awesome if you could please fill out a feedback form here:

[Feedback](https://forms.office.com/Pages/ResponsePage.aspx?id=v4j5cvGGr0GRqy180BHbR_l_utPQfhRCt2OTUw0KODVUQjBLMDVGMzBINjBDRE02VkROQVZaOTZFSy4u)