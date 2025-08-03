# Cranberry waffles

A testing target to fuzz against ruby.

```
Prism -> Lower to Cranelift -> clif2wasm -> Browser execution
                    -> Run natively on rust
```

This is a research project to see where the ruby spec might be vague to
lower, given the same parser (prism), but a different backend
(cranelift). We should be able to test against different targets. The
Browser execution side is for sharing links (like compiler explorer does
for testing compilers).

Some research questions:

Q1: is `ruby/spec` concrete enough to distinguish plausibly correct but
incorrect implementations of ruby constructs?

Q2: Can metamorphic tests identify surprising behavior within CRuby that
runs counter to documented behavior?

Q3: Can testing these semantics through differential testing produce
clarifications to the written ruby spec as well as the automated test
suite (`ruby/spec`)?
