# Changelog

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
