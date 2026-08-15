# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow Home Assistant calendar versioning (`YYYY.M.PATCH`)
and [Conventional Commits](https://www.conventionalcommits.org/).

## [2026.8.6](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.5...2026.8.6) - 2026-08-15

### Bug Fixes

- Start the GHCR add-on image through `klar-entry.sh` so HTTP and Wyoming bind `0.0.0.0` and Home Assistant can reach the engine

## [2026.8.5](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.4...2026.8.5) - 2026-08-15

### Bug Fixes

- Skip Adaptive Lighting switches when expanding Licht, so room follow-ups do not flip adapt or sleep modes
- Take GitHub release notes from CHANGELOG.md instead of `git-cliff --latest` after a `chore(release):` squash

## [2026.8.4](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.3...2026.8.4) - 2026-08-15

### Security

- Require loopback or token for parse and GET APIs; bind localhost by default; keep addon ports off the host
- Fail engine download without a SHA-256 digest
- Skip LLM fallback when the agent advertises Assist tools
- Filter unexposed entities in resolve, compound, roles, timers, and follow-ups

### Features

- Speak clarify and vacuum replies from friendly names
- Reload the home graph when HA registries change
- Persist custom sentences in the overlay; cap sessions at 256

### Miscellaneous

- Split lang, parse, and registry modules under 500 lines
- Raise English smoke to 99%; add Wyoming, digest, speech, and fallback tests
- Wait for rustfmt and clippy before tagging a release

## [2026.8.3](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v2026.8.2...2026.8.3) - 2026-08-15



### Features

- role tags, natural speech, and CalVer tags without v([2a90d58](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/2a90d58))

## [2026.8.2](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v2026.8.1...v2026.8.2) - 2026-08-15



### Bug Fixes

- start release checks from the prepare job([94433cf](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/94433cf14971c61a5dfe0492643242ab725792c6))

- land release commits without an Actions pull request([4de314a](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/4de314a631984ce1a41c3a99c8cece92c81084ae))

## [2026.8.1](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v2026.8.0...v2026.8.1) - 2026-08-15



### Miscellaneous

- run rustfmt in CI([77e0c95](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/77e0c95c7671af99e55f2b55ec99c943fe55d4e2))

## [2026.8.0](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v0.1.10...v2026.8.0) - 2026-08-15



### Bug Fixes

- store Assist personality in the Home Assistant integration (#30)([27b4846](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/27b48467b7e7accef7ffa5f9d47335e33eb22771))

- keep room-scoped all-lights and run the parsed entity (#29)([ad298e4](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/ad298e4d8f2ac3ddf1965e4d5e3191adb8f4c0ec))


### Miscellaneous

- switch releases to Home Assistant calendar versions (#31)([e099c83](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/e099c8388927f995fa4ad63337c6f4617af91b32))

## [0.1.10](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v0.1.9...v0.1.10) - 2026-08-15



### Bug Fixes

- keep und-names in questions and prefer the outlet (#27)([d8d5585](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/d8d5585027d21fb0d2636cfe6d4129ec61309b54))

## [0.1.9](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v0.1.8...v0.1.9) - 2026-08-15



### Bug Fixes

- pass the device name so Assist can run entity-only intents (#26)([904cd85](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/904cd856aebd043a63dabc184333b9473497fd43))

- do not match aus to the Alles-aus scene (#24)([f3a3bd6](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/f3a3bd622bf668c70ee8c9a9d3205eb386a3d92b))

## [0.1.8](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v0.1.7...v0.1.8) - 2026-08-15



### Bug Fixes

- keep one target and replay pronoun follow-ups (#22)([50a2ca2](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/50a2ca285881d57f16f968917ad0d8d7472e828d))

## [0.1.7](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v0.1.6...v0.1.7) - 2026-08-15



### Bug Fixes

- speak German states and keep the web UI personality on Assist (#20)([f322059](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/f32205998c86a1d35e21d75cb1bac9084b686d18))

## [0.1.6](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v0.1.5...v0.1.6) - 2026-08-15



### Features

- map leftover Assist devices in the web UI (#18)([bc85ce4](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/bc85ce45a03bd40d942b2dc2d43c81487017e81c))

## [0.1.5](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v0.1.4...v0.1.5) - 2026-08-15



### Bug Fixes

- inherit Hue room areas so Schlafzimmerlicht hits the Kugel([2ee34c8](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/2ee34c863db6d8e310eeff259131848056cd6597))

## [0.1.4](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v0.1.3...v0.1.4) - 2026-08-15



### Bug Fixes

- resolve live Wohnung lights, names, and English phrases([8fa3318](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/8fa331880af731590b7ffae678adfc222d35a972))

## [0.1.3](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v0.1.2...v0.1.3) - 2026-08-15



### Bug Fixes

- cancel timers and speak climate temperature (#12)([3449b33](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/3449b338232d885039cc8d26ce6382797a8979a4))

## [0.1.2](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v0.1.1...v0.1.2) - 2026-08-15



### Bug Fixes

- start Assist timers and match the real shopping list (#10)([e3e6de2](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/e3e6de2e16e0678681e1565f49169c2526765a5f))

## [0.1.1](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/v0.1.0...v0.1.1) - 2026-08-15



### Bug Fixes

- German and English Assist speech and rooms (#8)([3b4da80](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/3b4da808fbf07152b60f65bd800337d08621878b))

- bilingual Assist conversation agent (#6)([04a806a](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/04a806af3359c951bc9c8b23ba5cff316471f434))


### Features

- publish Docker images and a Home Assistant add-on repository([e62a715](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/e62a71537fcea8551cb6ecab60b9f4956f7cfc9e))


### Miscellaneous

- open a release PR instead of pushing version bumps to main([f38a784](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/f38a784c1cdbe0acfee2ddfcd4fc52b0bc5b8352))

- add CODEOWNERS, a PR template, and read-only CI permissions([7429d26](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/7429d2609a968ba4059f335ebb6acd1677d33440))

## [0.1.0](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/releases/tag/v0.1.0) - 2026-08-14



### Documentation

- shrink the README logo so it fits the HACS info panel([ea8e596](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/ea8e5964c74c462a7394537a8a9920092d2ba4be))

- use a markdown image for the logo so HACS can render it([82b0cd4](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/82b0cd4482ffe841886b6a192e612751c462e4c3))

- load the README logo from an absolute URL so HACS can show it([221c2c4](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/221c2c4b1961909e59167731c3ca20faed5b7608))


### Features

- start the Klar engine from the Home Assistant integration([b757353](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/b757353758d1dbe221477ef2b9f3b9aba72610f5))

- add HACS install for the Home Assistant integration([26e6916](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/26e6916b5aa494c2c0e8f495b2b9cc8f3d3e5a15))


### Miscellaneous

- bump checkout, artifact, and gh-release actions([dbf9171](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/dbf9171c913256658580bd20045168a568dc5959))

- generate changelogs and releases with git-cliff([a3cb586](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/a3cb5862c1a448f1312cd41b970e97bf5d222223))


### Other

- bump tower-http from 0.6 to 0.7([3abad97](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/3abad97e4ddde88df77c2ea6b615f0bbb33ecec7))

- Update dependabot.yml([3edeb08](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/3edeb087198c5df40c29022235de65a652dc04b8))

- Publish Klar NLU with CI, security scans, and multi-arch builds.([e4801a5](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/e4801a53878c5b65a8cf7f940a0f8283cc692e81))

<!-- generated by git-cliff -->
