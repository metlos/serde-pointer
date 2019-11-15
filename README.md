# serde-pointer

[![Build Status](https://api.travis-ci.org/metlos/serde-pointer.svg?branch=master)](https://travis-ci.org/metlos/serde-pointer)
[![Latest Version](https://img.shields.io/crates/v/serde-pointer.svg)](https://crates.io/crates/serde-pointer)
[![Code Coverage](https://codecov.io/gh/metlos/serde-pointer/branch/master/graph/badge.svg)](https://codecov.io/gh/metlos/serde-pointer)

Builds on top of [serde-value](http://arcnmx.github.io/serde-value/serde_value/) and provides a way of finding values in the intermediate representation provided by `serde-value` using the JSON pointers.

It is not tied to JSON though (apart from the JSON pointer syntax) and works generically over anything that `serde-value` can represent.
