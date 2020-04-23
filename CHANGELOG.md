
<a name="v0.2.0"></a>
## [v0.2.0](https://github.com/Kogia-sima/rust-covfix/compare/v0.1.1...v0.2.0) (2020-04-21)

### Breaking Changes

* Change starting line number (make 1-indexed)

### Bug Fixes

* Allow empty strings for --rules option
* Avoid panicking when source file is shorter than coverage record
* Fix line number updating of LoopRule
* Fix Runtime Error caused by not updating current_line

### Features

* Fix coverage based on syntax


<a name="v0.1.1"></a>
## [v0.1.1](https://github.com/Kogia-sima/rust-covfix/compare/v0.1.0...v0.1.1) (2020-01-13)

### Bug Fixes

* Fix descriptions for cli option


<a name="v0.1.0"></a>
## [v0.1.0](https://github.com/Kogia-sima/rust-covfix/compare/v0.0.2...v0.1.0) (2020-01-08)

### Breaking Changes

* Make BranchCoverage.line_number non-optional
* Make coverage count and taken optional

### Bug Fixes

* Fix problem that some line are not ignored.


<a name="v0.0.2"></a>
## [v0.0.2](https://github.com/Kogia-sima/rust-covfix/compare/v0.0.1...v0.0.2) (2019-12-24)

### Breaking Changes

* Change type of line_coverage.count

### Bug Fixes

* Ignore tests module correctly

