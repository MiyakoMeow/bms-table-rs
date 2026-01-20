# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.12.0](https://github.com/MiyakoMeow/bms-table-rs/compare/v0.11.0...v0.12.0) (2025-12-27)


### ⚠ BREAKING CHANGES

* **fetch:** renewed API ([#44](https://github.com/MiyakoMeow/bms-table-rs/issues/44))
* **reqwest:** use reqwest::IntoUrl ([#27](https://github.com/MiyakoMeow/bms-table-rs/issues/27))

### Features

* **reqwest:** use reqwest::IntoUrl ([#27](https://github.com/MiyakoMeow/bms-table-rs/issues/27)) ([2d7629a](https://github.com/MiyakoMeow/bms-table-rs/commit/2d7629a62378356da860c3bfff694cbe6ad41010))


### Bug Fixes

* add linter ([#37](https://github.com/MiyakoMeow/bms-table-rs/issues/37)) ([cf9dcf3](https://github.com/MiyakoMeow/bms-table-rs/commit/cf9dcf3c75ed02b87b589eeb692ccb7d66630977))
* clippy ([#42](https://github.com/MiyakoMeow/bms-table-rs/issues/42)) ([ebd689b](https://github.com/MiyakoMeow/bms-table-rs/commit/ebd689ba151b957d7d95d438d204406ac1f03119))
* **doc:** Option ([#38](https://github.com/MiyakoMeow/bms-table-rs/issues/38)) ([19043ad](https://github.com/MiyakoMeow/bms-table-rs/commit/19043adc40e8e2cbe344885df5f81d657435356f))
* **error:** duplicated message ([#24](https://github.com/MiyakoMeow/bms-table-rs/issues/24)) ([28e1838](https://github.com/MiyakoMeow/bms-table-rs/commit/28e18388806ce616fd8b3335187a8d58a88db703))
* **fetch:** small fix & tips ([#39](https://github.com/MiyakoMeow/bms-table-rs/issues/39)) ([8ebbe1e](https://github.com/MiyakoMeow/bms-table-rs/commit/8ebbe1e561d892b799bff37c75f33fc5e53db1e6))


### Miscellaneous Chores

* change release ([#43](https://github.com/MiyakoMeow/bms-table-rs/issues/43)) ([6548fd2](https://github.com/MiyakoMeow/bms-table-rs/commit/6548fd2726641dde1e6f5e9a7a8dbe7d92f17477))


### Code Refactoring

* **fetch:** renewed API ([#44](https://github.com/MiyakoMeow/bms-table-rs/issues/44)) ([65790da](https://github.com/MiyakoMeow/bms-table-rs/commit/65790da415a6dbb61c6f897be06869c5804341a2))

## [0.11.0](https://github.com/MiyakoMeow/bms-table-rs/compare/v0.10.4...v0.11.0) (2025-11-16)


### ⚠ BREAKING CHANGES

* add fallback ([#23](https://github.com/MiyakoMeow/bms-table-rs/issues/23))
* index -> list ([#22](https://github.com/MiyakoMeow/bms-table-rs/issues/22))

### rename

* index -&gt; list ([#22](https://github.com/MiyakoMeow/bms-table-rs/issues/22)) ([e106dfa](https://github.com/MiyakoMeow/bms-table-rs/commit/e106dfa31fb86c816bc865fb86ad536c3641032f))


### Features

* add fallback ([#23](https://github.com/MiyakoMeow/bms-table-rs/issues/23)) ([8e9eac4](https://github.com/MiyakoMeow/bms-table-rs/commit/8e9eac4c3716a372e91fac08870626e120f6a7f1))
* **reqwest:** use serde when possible ([#20](https://github.com/MiyakoMeow/bms-table-rs/issues/20)) ([252d426](https://github.com/MiyakoMeow/bms-table-rs/commit/252d4268e6fe0ab8df027d7bf1c011ef905036d2))

## [0.10.4](https://github.com/MiyakoMeow/bms-table-rs/compare/v0.10.3...v0.10.4) (2025-11-12)


### Bug Fixes

* **doc:** clearer describe ([#17](https://github.com/MiyakoMeow/bms-table-rs/issues/17)) ([548abee](https://github.com/MiyakoMeow/bms-table-rs/commit/548abeeec324ef29e0e028e55d6a1accb0198ef8))

## [0.10.3](https://github.com/MiyakoMeow/bms-table-rs/compare/v0.10.2...v0.10.3) (2025-11-12)


### Bug Fixes

* **README:** accepted structure ([#15](https://github.com/MiyakoMeow/bms-table-rs/issues/15)) ([05f1709](https://github.com/MiyakoMeow/bms-table-rs/commit/05f1709764e2508c246dc1dd1f1fd1539821020c))

## [0.10.2](https://github.com/MiyakoMeow/bms-table-rs/compare/v0.10.1...v0.10.2) (2025-11-12)


### Bug Fixes

* **test:** lib.rs offline test ([#13](https://github.com/MiyakoMeow/bms-table-rs/issues/13)) ([4cdc025](https://github.com/MiyakoMeow/bms-table-rs/commit/4cdc0257f719a322d7306f272cc37e16a6d9e013))

## [0.10.1](https://github.com/MiyakoMeow/bms-table-rs/compare/v0.10.0...v0.10.1) (2025-11-12)


### Features

* **reqwest:** client act like a browser ([#10](https://github.com/MiyakoMeow/bms-table-rs/issues/10)) ([c130ce8](https://github.com/MiyakoMeow/bms-table-rs/commit/c130ce84acbf3dbc69597fd917ab00c6c8b2f814))


### Bug Fixes

* **ci:** renew renovate config ([#7](https://github.com/MiyakoMeow/bms-table-rs/issues/7)) ([72e47fd](https://github.com/MiyakoMeow/bms-table-rs/commit/72e47fd0d0983c803135f0a057f56a3877af05c0))
* **ci:** schedule ([fc045b7](https://github.com/MiyakoMeow/bms-table-rs/commit/fc045b70bfad8f2d150212c0f4bde862655244ad))
* **ci:** use helper bot ([#9](https://github.com/MiyakoMeow/bms-table-rs/issues/9)) ([f7345ab](https://github.com/MiyakoMeow/bms-table-rs/commit/f7345abf96dedf7aff8a21206ae88f2d42c0de6f))


### Miscellaneous Chores

* change release to 0.10.1 ([#11](https://github.com/MiyakoMeow/bms-table-rs/issues/11)) ([824b648](https://github.com/MiyakoMeow/bms-table-rs/commit/824b6482e7a4b6332001a5c99c08e3d1b9560c82))

## [0.10.0](https://github.com/MiyakoMeow/bms-table-rs/compare/v0.9.1...v0.10.0) (2025-11-11)


### ⚠ BREAKING CHANGES

* add json_url to BmsTableRaw, feature check

### Features

* add json_url to BmsTableRaw, feature check ([7f116d3](https://github.com/MiyakoMeow/bms-table-rs/commit/7f116d3fc92614c4bb419e72781cd1f1e0402098))


### Bug Fixes

* fmt ([8cc8cf2](https://github.com/MiyakoMeow/bms-table-rs/commit/8cc8cf2ca91242ebde854381d48856b9bd021614))
* **README:** badges ([0dbbaaf](https://github.com/MiyakoMeow/bms-table-rs/commit/0dbbaaf35c48c48fe211d7d54e49d4d575690f1e))
* **README:** badges ([771f5a6](https://github.com/MiyakoMeow/bms-table-rs/commit/771f5a645f1ee12dc88cbb8c7033865c118f3ed9))
* **README:** badges style ([fdbed95](https://github.com/MiyakoMeow/bms-table-rs/commit/fdbed954fe69ec2641742f9a4220a47ab3ea36f9))


### Miscellaneous Chores

* change release to 0.10.0 ([7079781](https://github.com/MiyakoMeow/bms-table-rs/commit/7079781d85f77c30e11efe4ae854394324718e65))

## [1.0.0](https://github.com/MiyakoMeow/bms-table-rs/compare/v0.9.1...v1.0.0) (2025-11-11)


### ⚠ BREAKING CHANGES

* add json_url to BmsTableRaw, feature check

### Features

* add json_url to BmsTableRaw, feature check ([7f116d3](https://github.com/MiyakoMeow/bms-table-rs/commit/7f116d3fc92614c4bb419e72781cd1f1e0402098))


### Bug Fixes

* fmt ([8cc8cf2](https://github.com/MiyakoMeow/bms-table-rs/commit/8cc8cf2ca91242ebde854381d48856b9bd021614))
* **README:** badges ([0dbbaaf](https://github.com/MiyakoMeow/bms-table-rs/commit/0dbbaaf35c48c48fe211d7d54e49d4d575690f1e))
* **README:** badges ([771f5a6](https://github.com/MiyakoMeow/bms-table-rs/commit/771f5a645f1ee12dc88cbb8c7033865c118f3ed9))
* **README:** badges style ([fdbed95](https://github.com/MiyakoMeow/bms-table-rs/commit/fdbed954fe69ec2641742f9a4220a47ab3ea36f9))

## [0.9.1](https://github.com/MiyakoMeow/bms-table-rs/compare/v0.9.0...v0.9.1) (2025-11-10)


### Features

* **reqwest:** make client more lenient ([dbed960](https://github.com/MiyakoMeow/bms-table-rs/commit/dbed960cb9f443b6e2da859a5f34ee03cda182ef))


### Bug Fixes

* **ci/publish:** spec toolchain ([03bd303](https://github.com/MiyakoMeow/bms-table-rs/commit/03bd30304034200dd3714cc942b385a0d315af89))
* **ci/release:** set type rust ([76a7f13](https://github.com/MiyakoMeow/bms-table-rs/commit/76a7f133f6b6c6af5627ab494860523e75d6fba4))


### Miscellaneous Chores

* change release to 0.9.1 ([b793e48](https://github.com/MiyakoMeow/bms-table-rs/commit/b793e48200d32859d3f6125da0a21b3b51b2db42))

## [0.9.0](https://github.com/MiyakoMeow/bms-table-rs/compare/v0.8.0...v0.9.0) (2025-11-10)


### ⚠ BREAKING CHANGES

* de_string_opt

### del

* de_string_opt ([38ba034](https://github.com/MiyakoMeow/bms-table-rs/commit/38ba03472ca7a135615e1dfd9a5e3a899aaeabdd))


### Features

* **reqwest:** clean control chars ([95ff735](https://github.com/MiyakoMeow/bms-table-rs/commit/95ff735f6650b2ef96faeda6b96cc326f45de4d2))


### Bug Fixes

* **de:** accepts number as level ([86685fd](https://github.com/MiyakoMeow/bms-table-rs/commit/86685fd2cf7bcb1beed68147e17414e1a9e146d5))
* tests ([441fbaa](https://github.com/MiyakoMeow/bms-table-rs/commit/441fbaaf70e48f195bfb8347b5153cd68bbd1601))


### Miscellaneous Chores

* release 0.9.0 ([d972bc1](https://github.com/MiyakoMeow/bms-table-rs/commit/d972bc18d6fb263cb226b754df03301966a92d3a))
