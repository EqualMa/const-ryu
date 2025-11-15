# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/EqualMa/const-ryu/compare/04b87ae111c40a9400f757259717d224f6f502ab...v0.1.0) - 2025-11-15

### Features

- Const api with [`Format`](https://docs.rs/const-ryu/0.1.0/const_ryu/struct.Format.html) and [`FormatFinite`](https://docs.rs/const-ryu/0.1.0/const_ryu/struct.FormatFinite.html).

### BREAKING CHANGES

- Package is renamed to `const-ryu`
- feature `no-panic` doesn't work anymore because `no_panic attribute on const fn is not supported`
- `rust-version = "1.83"`
