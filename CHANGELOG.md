# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]
## [0.1.0-alpha34] - 2024-09-30

- improve perf: `tagger` will cache the loaded images in memory

## [0.1.0-alpha33] - 2024-09-30

- improve perf and fix bug: `tagger pick`

## [0.1.0-alpha32] - 2024-09-29

- enhance: `tagger pick` page ops
- refact with `Widget`

## [0.1.0-alpha31] - 2024-09-28

- improve: a more efficient image picker
- bump deps

## [0.1.0-alpha30] - 2024-09-28

- fix: `data_loader open_image` now canonicalizes path before opening
- fix: `Image Component` now canonicalizes path before opening
- fix: `tagger observe` panic if the height of the terminal is too small
- improve: default ratio of `tagger divide` is 9:1 now

## [0.1.0-alpha29] - 2024-09-25

- remove unnecessary layers in model
- fix score: a vital bug of data loader, it used to resize images in a wrong way

## [0.1.0-alpha28] - 2024-09-25

- improve: real-time output of `score predict` tty mode
- improve: better default arguments for `score predict`

## [0.1.0-alpha27] - 2024-09-24

- fix score: dataloader panic if image is zero size
- improve score: preset more default arguments
- improve score: more metrics to show
- fix tagger observe: panic if height of terminal is zero
- fix tagger pick: now abusolute path is used for softlink
- enhance tagger pick: add hardlink support

## [0.1.0-alpha26] - 2024-09-24

- generate auto complete scripts
- enhance tagger: add `tagger divide` subcommand
- enhance tagger: add `tagger observe` subcommand

## [0.1.0-alpha25] - 2024-09-22

- improve: show path and error if failed to load image

## [0.1.0-alpha24] - 2024-09-22

- fix: memory leak of `Matrix`'s inner map keys

## [0.1.0-alpha23] - 2024-09-21

- counter not increase when recovering from cache

## [0.1.0-alpha22] - 2024-09-21

- fix picker: cache won't be saved after finishing
- improve picker: finish screen will show when finishing

## [0.1.0-alpha21] - 2024-09-21

- fix: a bug when handling duplicate file name

## [0.1.0-alpha20] - 2024-09-19

- improve: allow pretained model to be used

## [0.1.0-alpha19] - 2024-09-18

- fix: score open image does not follow symlink

## [0.1.0-alpha18] - 2024-09-18

- use FilterType::Gaussian instead for better resolution

## [0.1.0-alpha17] - 2024-09-18

- fix picker: quiting will mis-cache the last image as dropped

## [0.1.0-alpha16] - 2024-09-17

- fix: no delete key on mac, use backspace instead

## [0.1.0-alpha15] - 2024-09-17

- tagger now support pick images

## [0.1.0-alpha14] - 2024-09-17

- cache relative path instead

## [0.1.0-alpha13] - 2024-09-17

- give up windows tagger support

## [0.1.0-alpha12] - 2024-09-17

- fix ci for windows

## [0.1.0-alpha11] - 2024-09-15

- fix windows setups script mistake
- try fix windows image display bug

## [0.1.0-alpha10] - 2024-09-15

- score MVP

## [0.1.0-alpha9] - 2024-09-14

- tagger MVP

## [0.1.0-alpha8] - 2024-09-14

- ci test

## [0.1.0-alpha7] - 2024-09-12

- ci test

## [0.1.0-alpha6] - 2024-09-12

- ci test

## [0.1.0-alpha5] - 2024-09-11

- ci test

## [0.1.0-alpha4] - 2024-09-10

- ci test

## [0.1.0-alpha3] - 2024-09-09

- ci test

## [0.1.0-alpha2] - 2024-09-05

- ci test

## [0.1.0-alpha1] - 2024-09-03

- ci test
