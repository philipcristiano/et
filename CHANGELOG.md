# Changelog

## [2.1.1](https://github.com/philipcristiano/et/compare/v2.1.0...v2.1.1) (2024-04-13)


### Bug Fixes

* Don't push url state for balance loads ([f7b271f](https://github.com/philipcristiano/et/commit/f7b271fb1c0d22887202c32ea878d958de755dd6))

## [2.1.0](https://github.com/philipcristiano/et/compare/v2.0.1...v2.1.0) (2024-04-13)


### Features

* Display aggregate value of a label ([1e0da28](https://github.com/philipcristiano/et/commit/1e0da28730f75bc3255646cf0ef80d0b13c384fc))
* Display connection errors ([34702e1](https://github.com/philipcristiano/et/commit/34702e12bc694dc9237ae753b9237df7e837fb30))

## [2.0.1](https://github.com/philipcristiano/et/compare/v2.0.0...v2.0.1) (2024-04-13)


### Bug Fixes

* Use search icons ([902b3e3](https://github.com/philipcristiano/et/commit/902b3e371ee2b35e4566095a82384f353504192f))

## [2.0.0](https://github.com/philipcristiano/et/compare/v1.0.0...v2.0.0) (2024-04-12)


### ⚠ BREAKING CHANGES

* breaking: use timestamptz / don't use natural keys

### Features

* Add labels to transactions ([1d71a2c](https://github.com/philipcristiano/et/commit/1d71a2c5ecc66eb6c30f75a3e45fca17b823cdef))
* Display transactions ([f3c3097](https://github.com/philipcristiano/et/commit/f3c3097c5b9ff9de9ab2b0cbd5617f6989d22c90))
* Filter by labels ([a1db550](https://github.com/philipcristiano/et/commit/a1db550911abc1df1ba24c4d517d13c3e04c3c84))
* Sync account transactions ([8bf3bbb](https://github.com/philipcristiano/et/commit/8bf3bbb7207bd5f49d7c9c9dd17bdee1c197be8b))
* Sync in background ([baa62d1](https://github.com/philipcristiano/et/commit/baa62d17893ee43abbd08c271edf1bdc739ef74f))
* Untie accounts/connections from users ([bc92076](https://github.com/philipcristiano/et/commit/bc9207633cdac1a7312df6e64aecbe54c47947d6))


### Bug Fixes

* Actually sync after time period ([dff136e](https://github.com/philipcristiano/et/commit/dff136ea4e8154d89f49378aec7203aa4b25a9a9))
* Add /_health endpoint ([1709400](https://github.com/philipcristiano/et/commit/1709400879dd9d5d95117d2af993821d0a7b6276))
* Allow deleting labels from transactions ([fc4a41c](https://github.com/philipcristiano/et/commit/fc4a41c10b15fe7ad74a63cb540d24de367d38c3))
* Attemp axum tracing again ([eb5bf70](https://github.com/philipcristiano/et/commit/eb5bf70d523d77abcad3ac9f5bb839c403ff857e))
* Attempt custom span ([fb553a9](https://github.com/philipcristiano/et/commit/fb553a973386fe995ad3ad97b26cf0fa96cc4f0d))
* breaking: use timestamptz / don't use natural keys ([9a3e5bf](https://github.com/philipcristiano/et/commit/9a3e5bf76fd19ee1574f31fdc08b67e5f60a0890))
* Clicking on transaction filters table ([3819426](https://github.com/philipcristiano/et/commit/3819426c9dd168a76c7de72e1a72bc9723a5b92b))
* **deps:** update rust crate anyhow to 1.0.82 ([b6f56f8](https://github.com/philipcristiano/et/commit/b6f56f8232a5cf1dd388b43ccba2473ee7902791))
* **deps:** update rust crate axum to 0.7.5 ([244ff4d](https://github.com/philipcristiano/et/commit/244ff4d7bda1b42e1cf5da2b8ec62c668b1c1f89))
* **deps:** update rust crate chrono to 0.4.37 ([fa5f1da](https://github.com/philipcristiano/et/commit/fa5f1da2522f15c5d11bfb027a1a387e3a6814ab))
* **deps:** update rust crate clap to 4.5.4 ([6a089cb](https://github.com/philipcristiano/et/commit/6a089cb5a3e0764340c35c2bbd5f5723a3ababa9))
* **deps:** update rust crate once_cell to 1.19.0 ([efd34ce](https://github.com/philipcristiano/et/commit/efd34cea295a9a11debaff8bf3331bd4c4e0d51e))
* **deps:** update rust crate service_conventions to 0.0.13 ([87f62af](https://github.com/philipcristiano/et/commit/87f62af9a014d2b956c1afadac092d51f872c0b3))
* **deps:** update rust crate service_conventions to 0.0.14 ([a564a62](https://github.com/philipcristiano/et/commit/a564a62de145ecc376a2151ccf75ffd667af24fa))
* **deps:** update rust crate sqlx to 0.7.4 ([8e03017](https://github.com/philipcristiano/et/commit/8e0301763087e23de748a5ea07b8d92e8b8323e6))
* **deps:** update rust crate tokio to 1.37.0 ([4ae0563](https://github.com/philipcristiano/et/commit/4ae056311e9d6098919e143529896acb7135998d))
* **deps:** update rust crate toml to 0.8.12 ([1bdecfd](https://github.com/philipcristiano/et/commit/1bdecfd949cd8d5ccbcc5db09929c660f4b9a5c0))
* Display transaction time if set ([216ff96](https://github.com/philipcristiano/et/commit/216ff960e92a70d52b0e6ea0af98ed1df89ff342))
* dockerfile: Remove duplicating build steps ([4584f66](https://github.com/philipcristiano/et/commit/4584f660910effa27f863e8074a2579852bf0269))
* Error on tailwind error ([42e9841](https://github.com/philipcristiano/et/commit/42e9841bdfdee64206dd526088504aecca5dc68b))
* Fix cargo lock package version ([c4b9180](https://github.com/philipcristiano/et/commit/c4b91803ab96bab6e3129386dccf68419878e6c2))
* Fix formatting ([e969ce9](https://github.com/philipcristiano/et/commit/e969ce91e46f9104d965b9b51170c74dead74f46))
* Include tracing for more queries ([12244f4](https://github.com/philipcristiano/et/commit/12244f45e4abf6dcbcc5e64c8ecb509ab5359d2c))
* labels: Remove duplicate #main ([5e73ed9](https://github.com/philipcristiano/et/commit/5e73ed9e5481ca5c2c6ce13a04ed0e7c136f2189))
* layout: Use tables for tabular data ([ce72632](https://github.com/philipcristiano/et/commit/ce726328d0faf4de63ec2e8ffbec669478a5ea82))
* logging: Don't log db credentials ([3d57c6d](https://github.com/philipcristiano/et/commit/3d57c6d7a4b99dfb2f29e9529723acdae43c7519))
* otel: Maybe better resource tags ([04c3937](https://github.com/philipcristiano/et/commit/04c39372bb2feab926f00f0b591338e96abbea4b))
* Revert "fix: Attemp axum tracing again" ([64f92da](https://github.com/philipcristiano/et/commit/64f92da4a7fedf9d827d89a16a03aeecd11e1431))
* Revert "fix: Attempt custom span" ([8f3ba90](https://github.com/philipcristiano/et/commit/8f3ba90d199f8aa2fd8bbed7659e20432ffc7a21))
* Revert "fix: Try to handle otel headers" ([677042d](https://github.com/philipcristiano/et/commit/677042ddca3135251d9e744878526f5a4a5b2af9))
* Run tx inserts concurrently ([a1528d6](https://github.com/philipcristiano/et/commit/a1528d6988897602222067bd9d28f96d68b571df))
* Save URL state when switching accounts ([8d746dc](https://github.com/philipcristiano/et/commit/8d746dc378291d2f3c11790eab73fdd5a08a9769))
* Simplify static file hosting ([b7f0ff5](https://github.com/philipcristiano/et/commit/b7f0ff5303b1f68fe2742232b6d14722390fddc0))
* sql: Use separate pool for spikey ops ([7aa7a89](https://github.com/philipcristiano/et/commit/7aa7a89d9058ee8c4b78d74652487af616761bfb))
* Start saving labels ([c573778](https://github.com/philipcristiano/et/commit/c573778b590a29dd7ce383cbd012dfcc6104b863))
* Start tracing some calls ([4da63c2](https://github.com/philipcristiano/et/commit/4da63c21a78b38ea0f7fc3c5435076d10e3529fe))
* style: Formatting in columns ([a1faac6](https://github.com/philipcristiano/et/commit/a1faac6a0b11d84e389c22809732a2b33a0c9377))
* tracing: Attempt again to get OTEL gRPC traces working ([fefd5b3](https://github.com/philipcristiano/et/commit/fefd5b39db063ee512f1503a04866fc3d34148d4))
* tracing: include deps for grpc over https ([c094be2](https://github.com/philipcristiano/et/commit/c094be20f2640c8bb62e5f4aaaf9c32e605fc0b1))
* tracing: Include TLS ([ab58695](https://github.com/philipcristiano/et/commit/ab58695e45eec17803b2130b55c11b5c21bbee71))
* Try to handle otel headers ([ca76fe0](https://github.com/philipcristiano/et/commit/ca76fe0c1ec7075ddcf85ffa727f45a894299d39))

## [0.6.0](https://github.com/philipcristiano/et/compare/v0.5.0...v1.0.0) (2024-04-11)


### ⚠ BREAKING CHANGES

* breaking: use timestamptz / don't use natural keys

### Features

* Add labels to transactions ([1d71a2c](https://github.com/philipcristiano/et/commit/1d71a2c5ecc66eb6c30f75a3e45fca17b823cdef))


### Bug Fixes

* Actually sync after time period ([dff136e](https://github.com/philipcristiano/et/commit/dff136ea4e8154d89f49378aec7203aa4b25a9a9))
* breaking: use timestamptz / don't use natural keys ([9a3e5bf](https://github.com/philipcristiano/et/commit/9a3e5bf76fd19ee1574f31fdc08b67e5f60a0890))
* Clicking on transaction filters table ([3819426](https://github.com/philipcristiano/et/commit/3819426c9dd168a76c7de72e1a72bc9723a5b92b))
* **deps:** update rust crate anyhow to 1.0.82 ([b6f56f8](https://github.com/philipcristiano/et/commit/b6f56f8232a5cf1dd388b43ccba2473ee7902791))
* Include tracing for more queries ([12244f4](https://github.com/philipcristiano/et/commit/12244f45e4abf6dcbcc5e64c8ecb509ab5359d2c))
* labels: Remove duplicate #main ([5e73ed9](https://github.com/philipcristiano/et/commit/5e73ed9e5481ca5c2c6ce13a04ed0e7c136f2189))
* Save URL state when switching accounts ([8d746dc](https://github.com/philipcristiano/et/commit/8d746dc378291d2f3c11790eab73fdd5a08a9769))
* Start saving labels ([c573778](https://github.com/philipcristiano/et/commit/c573778b590a29dd7ce383cbd012dfcc6104b863))

## [0.5.0](https://github.com/philipcristiano/et/compare/v0.4.2...v0.5.0) (2024-04-04)


### Features

* Sync in background ([baa62d1](https://github.com/philipcristiano/et/commit/baa62d17893ee43abbd08c271edf1bdc739ef74f))


### Bug Fixes

* Display transaction time if set ([216ff96](https://github.com/philipcristiano/et/commit/216ff960e92a70d52b0e6ea0af98ed1df89ff342))
* Error on tailwind error ([42e9841](https://github.com/philipcristiano/et/commit/42e9841bdfdee64206dd526088504aecca5dc68b))
* style: Formatting in columns ([a1faac6](https://github.com/philipcristiano/et/commit/a1faac6a0b11d84e389c22809732a2b33a0c9377))

## [0.4.2](https://github.com/philipcristiano/et/compare/v0.4.1...v0.4.2) (2024-04-04)


### Bug Fixes

* Add /_health endpoint ([1709400](https://github.com/philipcristiano/et/commit/1709400879dd9d5d95117d2af993821d0a7b6276))
* Attempt custom span ([fb553a9](https://github.com/philipcristiano/et/commit/fb553a973386fe995ad3ad97b26cf0fa96cc4f0d))
* **deps:** update rust crate service_conventions to 0.0.13 ([87f62af](https://github.com/philipcristiano/et/commit/87f62af9a014d2b956c1afadac092d51f872c0b3))
* **deps:** update rust crate service_conventions to 0.0.14 ([a564a62](https://github.com/philipcristiano/et/commit/a564a62de145ecc376a2151ccf75ffd667af24fa))
* Fix formatting ([e969ce9](https://github.com/philipcristiano/et/commit/e969ce91e46f9104d965b9b51170c74dead74f46))
* Revert "fix: Attemp axum tracing again" ([64f92da](https://github.com/philipcristiano/et/commit/64f92da4a7fedf9d827d89a16a03aeecd11e1431))
* Revert "fix: Attempt custom span" ([8f3ba90](https://github.com/philipcristiano/et/commit/8f3ba90d199f8aa2fd8bbed7659e20432ffc7a21))

## [0.4.1](https://github.com/philipcristiano/et/compare/v0.4.0...v0.4.1) (2024-04-03)


### Bug Fixes

* Attemp axum tracing again ([eb5bf70](https://github.com/philipcristiano/et/commit/eb5bf70d523d77abcad3ac9f5bb839c403ff857e))

## [0.4.0](https://github.com/philipcristiano/et/compare/v0.3.8...v0.4.0) (2024-04-03)


### Features

* Untie accounts/connections from users ([bc92076](https://github.com/philipcristiano/et/commit/bc9207633cdac1a7312df6e64aecbe54c47947d6))


### Bug Fixes

* Run tx inserts concurrently ([a1528d6](https://github.com/philipcristiano/et/commit/a1528d6988897602222067bd9d28f96d68b571df))
* sql: Use separate pool for spikey ops ([7aa7a89](https://github.com/philipcristiano/et/commit/7aa7a89d9058ee8c4b78d74652487af616761bfb))

## [0.3.8](https://github.com/philipcristiano/et/compare/v0.3.7...v0.3.8) (2024-04-02)


### Bug Fixes

* Revert "fix: Try to handle otel headers" ([677042d](https://github.com/philipcristiano/et/commit/677042ddca3135251d9e744878526f5a4a5b2af9))

## [0.3.7](https://github.com/philipcristiano/et/compare/v0.3.6...v0.3.7) (2024-04-02)


### Bug Fixes

* Try to handle otel headers ([ca76fe0](https://github.com/philipcristiano/et/commit/ca76fe0c1ec7075ddcf85ffa727f45a894299d39))

## [0.3.6](https://github.com/philipcristiano/et/compare/v0.3.5...v0.3.6) (2024-04-02)


### Bug Fixes

* layout: Use tables for tabular data ([ce72632](https://github.com/philipcristiano/et/commit/ce726328d0faf4de63ec2e8ffbec669478a5ea82))

## [0.3.5](https://github.com/philipcristiano/et/compare/v0.3.4...v0.3.5) (2024-04-01)


### Bug Fixes

* logging: Don't log db credentials ([3d57c6d](https://github.com/philipcristiano/et/commit/3d57c6d7a4b99dfb2f29e9529723acdae43c7519))
* Start tracing some calls ([4da63c2](https://github.com/philipcristiano/et/commit/4da63c21a78b38ea0f7fc3c5435076d10e3529fe))

## [0.3.4](https://github.com/philipcristiano/et/compare/v0.3.3...v0.3.4) (2024-03-31)


### Bug Fixes

* otel: Maybe better resource tags ([04c3937](https://github.com/philipcristiano/et/commit/04c39372bb2feab926f00f0b591338e96abbea4b))

## [0.3.3](https://github.com/philipcristiano/et/compare/v0.3.2...v0.3.3) (2024-03-31)


### Bug Fixes

* tracing: Attempt again to get OTEL gRPC traces working ([fefd5b3](https://github.com/philipcristiano/et/commit/fefd5b39db063ee512f1503a04866fc3d34148d4))

## [0.3.2](https://github.com/philipcristiano/et/compare/v0.3.1...v0.3.2) (2024-03-31)


### Bug Fixes

* tracing: Include TLS ([ab58695](https://github.com/philipcristiano/et/commit/ab58695e45eec17803b2130b55c11b5c21bbee71))

## [0.3.1](https://github.com/philipcristiano/et/compare/v0.3.0...v0.3.1) (2024-03-31)


### Bug Fixes

* tracing: include deps for grpc over https ([c094be2](https://github.com/philipcristiano/et/commit/c094be20f2640c8bb62e5f4aaaf9c32e605fc0b1))

## [0.3.0](https://github.com/philipcristiano/et/compare/v0.2.0...v0.3.0) (2024-03-31)


### Features

* Display transactions ([f3c3097](https://github.com/philipcristiano/et/commit/f3c3097c5b9ff9de9ab2b0cbd5617f6989d22c90))


### Bug Fixes

* **deps:** update rust crate toml to 0.8.12 ([1bdecfd](https://github.com/philipcristiano/et/commit/1bdecfd949cd8d5ccbcc5db09929c660f4b9a5c0))
* Simplify static file hosting ([b7f0ff5](https://github.com/philipcristiano/et/commit/b7f0ff5303b1f68fe2742232b6d14722390fddc0))

## [0.2.0](https://github.com/philipcristiano/et/compare/v0.1.0...v0.2.0) (2024-03-31)


### Features

* Sync account transactions ([8bf3bbb](https://github.com/philipcristiano/et/commit/8bf3bbb7207bd5f49d7c9c9dd17bdee1c197be8b))


### Bug Fixes

* **deps:** update rust crate chrono to 0.4.37 ([fa5f1da](https://github.com/philipcristiano/et/commit/fa5f1da2522f15c5d11bfb027a1a387e3a6814ab))
* **deps:** update rust crate clap to 4.5.4 ([6a089cb](https://github.com/philipcristiano/et/commit/6a089cb5a3e0764340c35c2bbd5f5723a3ababa9))
* **deps:** update rust crate once_cell to 1.19.0 ([efd34ce](https://github.com/philipcristiano/et/commit/efd34cea295a9a11debaff8bf3331bd4c4e0d51e))
* **deps:** update rust crate sqlx to 0.7.4 ([8e03017](https://github.com/philipcristiano/et/commit/8e0301763087e23de748a5ea07b8d92e8b8323e6))
* **deps:** update rust crate tokio to 1.37.0 ([4ae0563](https://github.com/philipcristiano/et/commit/4ae056311e9d6098919e143529896acb7135998d))

## 0.1.0 (2024-03-30)


### Bug Fixes

* **deps:** update rust crate axum to 0.7.5 ([244ff4d](https://github.com/philipcristiano/et/commit/244ff4d7bda1b42e1cf5da2b8ec62c668b1c1f89))
