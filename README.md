# pyglob
A python module to do wildcard pattern matching, written in rust

Please don't actually use it, it's slower than `fnmatch`, since it uses `re`, and whatever crazy optimisations are done to make it fast are better than this package. This package is about 2x slower than `re`, so there's really no benefit to using it.
