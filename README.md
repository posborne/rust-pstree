pstree in Rust
==============

This repository contains a simple implementation of the Unix pstree
program in Rust.  This is mostly done as a learning exercise and port
of a [C implementation I
wrote](https://github.com/posborne/linux-programming-interface-exercises/blob/6b1ae2357d7c73378a56e2d7b499b4ab49c4452f/12-system-and-process-information/pstree.c)
as an exercise in the Excellent book [The Linux Programming Interface
(TLPI)](http://man7.org/tlpi/).

The implementation here does not and is not intended to fully match
the implementation of pstree that you might find on your computer and
is not intended to replace it.

Build and Run It
-----------------

You will need to install the latest version of rust.  This has been tested
with the latest nightlies as of March 3, 2015 (1.0.0 alpha).

    $ cargo build
    $ ./target/pstree

Notes From Implementing
-----------------------

As noted, implementing this program in Rust was very much a learning
exercise and a somewhat painful one at that.  Here's some useful
things I picked up along the way that might be helpful for other
aspiring Rustafarians.

### The Compiler is Probably Right

There are things you can get away with in languages like C that are
either unsafe or not provably safe.  Rust is a safe language (by
default) and it is useful to try to think in terms of the compiler
being able to reason about your code.

With rust-pstree, one thing that I found I was not thinking deep
enough was ownership of data associated with each process.  Rust is
not a managed language, so it must be clear who owns and what the
lifetime of each piece of memory is for your code to compile.  This
takes some getting used to but starts to make sense over time.

As my work progressed, I started to be able to ask the right
questions.  This yielded much better results when asking for help on
IRC.  For instance, consider the following function:

```rust
fn populate_node(node : &mut ProcessTreeNode, records: &Vec<ProcessRecord>) {
    // populate the node by finding its children... recursively
    let pid = node.record.pid; // avoid binding node as immutable in closure
    for record in records.iter().filter(|record| record.ppid == pid) {
        let mut child = ProcessTreeNode::new(record);
        populate_node(&mut child, records);
        node.children.push(child);
    }
}
```

At first, a lot of things look weird here.  Now, I can look at this
and read through the code making the following obervations:

1. There is a function called `populate_node` that takes a mutable borrowed
   reference to a `ProcessTreeNode` and an immutable borrowed
   reference to a `Vec` of `ProccesRecord`s and returns nothing.  From
   this I know that the provided node may change but nothing else will
   including elements within the records vector.

2. First, there is an odd line with a comment.  It says `avoid binding
   node as immutable in closure`.  This change was made because we
   needed to make reference to the node's `pid` within the for loop
   but the compiler was preventing us from doing so through the node
   within the closure for the `filter` expression.

   Without this change, we receive the following error:
   ```
   pstree.rs:121:9: 121:22 error: cannot borrow `node.children` as mutable because `node` is also borrowed as immutable
   pstree.rs:121         node.children.push(child);
                          ^~~~~~~~~~~~~
   pstree.rs:118:41: 118:80 note: previous borrow of `node` occurs here due to use in closure; the immutable borrow prevents subsequent moves or mutable borrows of `node` until the borrow ends
   pstree.rs:118     for record in records.iter().filter(|record| record.ppid == node.record.pid) {
                                                          ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
   pstree.rs:123:2: 123:2 note: previous borrow ends here
   pstree.rs:118     for record in records.iter().filter(|record| record.ppid == node.record.pid) {
   pstree.rs:119         let mut child = ProcessTreeNode::new(record);
   pstree.rs:120         populate_node(&mut child, records);
   pstree.rs:121         node.children.push(child);
   pstree.rs:122     }
   pstree.rs:123 }
                 ^
   ```
    
   Well, that looks pretty confusing.  The 3rd line of the error,
   however, gives us a key insight.  Node is first borrowed as
   immutable implicitly by the closure and Rust's lexical scoping
   rules.  The life of that closure will be to the end of the for loop
   block.  This would usually not be a problem, but accessing the
   record of the node involves borrowing a reference to the node
   children.  Since `node` is mutable in this function, we are trying
   to obtain a mutable borrow within a closure where node is borrowed
   as immutable.  As this violates the immutability constraint, the
   compiler balks.

   We fix by storing an immutable copy of the pid prior to the
   creation of the closure.  Problem. solved.  All I can say, is that
   you get much better at reading the compiler's errors and fixing
   them quickly as you go on.

3. The rest of the function (in the body of the for loop), reads
   pretty clearly.  We have filtered all the records so that we are
   iterating over those records who are children of the node we are
   populating.  We create a new node for each of these children and
   populate it (recursive call).  Finally, we push the child into our
   list, taking ownership of it (e.g. The vec owns the child, the Node
   owns the vec, the tree owns the root node, and some scope (stack)
   owns the Tree.  It is pretty much what you would do in C/C++, but
   the compiler checks _much_ more for you, ensuring greater safety.

### The Best way to Learn is by Reading Other Code

At several points while implementing, I got stuck.  I found that
reading other code in the Rust standard library and in the community
often led to breakthroughs in my thinking.  Some of this was just
noticing a standard library function I had missed before (Oh, there is
an `iter_mut()` as well as an `iter()` on `Vec`?  Other times, it was
just picking up the 'rusty' way of structuring a program.

I can't claim to be an expert or to have generated anything good, but
reading is learning.  Also, viewing the change for some files over
time and the related PRs/discussion can be very interesting and
enlightening.

### Use IRC

I got good help when really stuck on the IRC channel.  Just have a
clear question ready to ask and code snippets ready to go.  The
community there wants to see you succeed.

Final Thoughts
--------------

This first experience with Rust has been somewhat frustrating but also
very enlightening.  My optimism that Rust could represent a real
threat to entrenched systems programming languages is not crushed.
The next few months as Rust moves toward 1.0 will be very
interesting.  I hope that the number of symbols in the language does
not increase much more.  I already find the two meanings for the `&`
operator to be confusing at times.  Explicit is better than implicit
in most cases.

Rust is not an 'easy' language to learn and I don't think it could be
while still doing what it needs to do.  Manual memory management is
not easy, but I believe that Rust provides a real service to
programmers and users by providing a language that greatly increases
the chances that a program that runs is actually safe.  I, for one,
have already spent more than enough time debugging race conditions and
resources leaks... Good riddance!

