# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow Home Assistant calendar versioning (`YYYY.M.PATCH`)
and [Conventional Commits](https://www.conventionalcommits.org/).

## [2026.8.36](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.35...2026.8.36) - 2026-08-19



### Other

- Merge pull request #110 from FABBricate-IT-Solutions/release/promote-staging([4cbc4fa](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/4cbc4facb509107250789472d0e96010abd566f1))

- Merge remote-tracking branch 'origin/main' into release/promote-staging([4bf2dec](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/4bf2dec4a89ff348bf228a1a9bf4652a49d5140d))

## [2026.8.35](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.34...2026.8.35) - 2026-08-18



### Bug Fixes

- options flow 400 on Configure (#108)([1e75383](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/1e753831d7c7d0e7a3409ffa9c2463affb794026))

## [2026.8.34](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.33...2026.8.34) - 2026-08-18



### Documentation

- bump rust from 1.85-bookworm to 1.97-bookworm (#104)([4d5bf80](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/4d5bf804d2bc7685d84fcc815cb7fa6837237cc5))


### Other

- bump uuid from 1.24.0 to 1.24.1 in the rust-patch group (#105)([828d1cb](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/828d1cb462e21644b63ab3779cc88a3fe41399c1))

## [2026.8.33](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.32...2026.8.33) - 2026-08-18



### Other

- Merge pull request #103 from FABBricate-IT-Solutions/release/promote-staging([4e5b3c6](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/4e5b3c682ee7f2dd3ca39a15ef0146dc6bfd58ce))

- main 2026.8.32 into staging for the stable cut([d572987](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/d57298791c7c8fc72ca9af38d24b5355af106ab0))

## [2026.8.32](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.31...2026.8.32) - 2026-08-18



### Features

- add a Stable/Staging switch on main([bb4f8f1](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/bb4f8f13117b79d1477bc288cc85fadcd5fd9c19))


### Other

- Merge pull request #97 from FABBricate-IT-Solutions/feat/simple-channel-switch-main([b106172](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/b1061725570f205f4df3ff1b32baeff5faf5b089))

## [2026.8.31](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.30...2026.8.31) - 2026-08-17



### Features

- phrase rules, journal tab, fuzzy compounds (#91)([c226a9d](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/c226a9dccb21e29bddc1429a2b40e89f462f8641))

## [2026.8.30](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.29...2026.8.30) - 2026-08-17



### Features

- policies, journal, NLU-RAG, and quieter UI (#88)([c651441](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/c65144115799bc02ca34b26c0240581e35b87c04))


### Miscellaneous

- nextest, faster PR checks, auto-release on main (#90)([5b9c1fa](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/5b9c1fac3bdcb89787be5803e5540ec6e4ec26fc))

- bump docker/setup-buildx-action from 3 to 4 (#87)([fa42574](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/fa42574b82ac8871a32bb6c7e3cac2c4f68a23da))

- bump actions/setup-node from 6 to 7 (#86)([77aa0bd](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/77aa0bd91526c96524737acdfa38bfdd5f57a460))

- bump docker/login-action from 3 to 4 (#85)([2256dd3](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/2256dd3c79e41f4faf443c6025bbf3b072183563))

## [2026.8.29](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.28...2026.8.29) - 2026-08-17



### Bug Fixes

- speak room climate from HA state when area get fails([142a7c8](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/142a7c8e1ae360c946623e6bffe627249ac31180))


### Other

- Merge pull request #83 from FABBricate-IT-Solutions/fix/area-climate-fallback([d28d7b3](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/d28d7b3092f7da85f1234168c76f542f464482b9))


### Testing

- mock entity_registry when loading dispatch([6342647](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/6342647c26681b8e24628c03a40ab285e7d120a2))

## [2026.8.28](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.27...2026.8.28) - 2026-08-16



### Bug Fixes

- let HA sync homes and answer temperature queries([ce107ae](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/ce107ae4a898325562fb419a89e657d6593a9b29))


### Other

- Merge pull request #81 from FABBricate-IT-Solutions/fix/live-query-sync([faf7cee](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/faf7ceeae83f57432d52d650eab64f5e4085b18d))

## [2026.8.27](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.26...2026.8.27) - 2026-08-16



### Bug Fixes

- accept V2 parse trace tokens from the engine([51126f4](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/51126f4ad85c6c2f5a15ca67b385a489ebb73ee1))


### Other

- Merge pull request #79 from FABBricate-IT-Solutions/fix/accept-trace-tokens([0457662](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/045766211c711907793908d788794cf4a58924e9))

## [2026.8.26](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.25...2026.8.26) - 2026-08-16



### Bug Fixes

- match rustfmt 1.97 line wrapping([b616d1d](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/b616d1da3bd990f7949e818d0eacdab3a8ced9d0))

- satisfy clippy 1.97 some_filter([bfd8b3a](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/bfd8b3ad81743290d3e8ae9e709e2245ba43678a))


### Features

- [**breaking**] replace the parse contract with V2 ParseOutcome([5f45302](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/5f45302e4ae0d864d04d5c4fff36c6e1b04e2cc9))


### Other

- Merge pull request #77 from FABBricate-IT-Solutions/feat/v2-nlu-platform([24bcb17](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/24bcb17ed76c326971db80b9002c1a63eae47759))

## [2026.8.25](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.24...2026.8.25) - 2026-08-16

### Bug Fixes

- harden Music Assistant targeting ([f995ccb](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/f995ccbcb7642f4e1e2b64b3d47bcf456ba33b83))

### Other

- Merge pull request #75 from FABBricate-IT-Solutions/fix/music-assistant-hardening ([4ff1deb](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/4ff1debc270605413e798a630acc6c9eb41c7ec3))

## [2026.8.24](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.23...2026.8.24) - 2026-08-16

### Features

- harden voice matching for ASR errors ([e99b683](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/e99b6830008468f6392db9a9e18844f94ac99e9b))

### Other

- Merge pull request #73 from FABBricate-IT-Solutions/feat/asr-fuzzy-matching ([f428df7](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/f428df71421e0ed4b8164abd3116a76018fa8be2))

## [2026.8.23](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.22...2026.8.23) - 2026-08-16



### Bug Fixes

- satisfy clippy for media parsing([f38048d](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/f38048d75331b4e52b539214bbec0024985dcb47))

- format Music Assistant voice control changes([240e577](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/240e577aba55a4418ced07d3527e8ed0866552f7))


### Features

- add Music Assistant voice control([70a943b](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/70a943b0670fbea0475012c7c0ae834f6ea54df5))


### Other

- Merge pull request #71 from FABBricate-IT-Solutions/feat/music-assistant-voice-control([9ef246e](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/9ef246e6b4a65720561850442915bf7b2e104494))

## [2026.8.22](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.21...2026.8.22) - 2026-08-16

### Bug Fixes

- keep graph nodes inside a scrollable room-based layout
- allow authenticated Home Assistant ingress to save UI settings
- count live Klar NLU traffic even when support-bundle recording is off

## [2026.8.21](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.20...2026.8.21) - 2026-08-16

### Bug Fixes

- include the built React UI in release container images

## [2026.8.20](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.19...2026.8.20) - 2026-08-16

### Features

- add a React operator UI with dashboard, assignment graph, calibration inbox, and Home Assistant sidebar ingress
- record optional support bundles and export Assist traffic as a dataset
- improve German status parsing for dative and plural light forms

### Testing

- add Assist replay datasets for live apartment voice queries

## [2026.8.19](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.18...2026.8.19) - 2026-08-15



### Other

- Merge pull request #66 from FABBricate-IT-Solutions/refactor/rust-structure-docs([bc89637](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/bc89637f56649d8b93f0e4cdb9c100dc2026913d))

- Merge branch 'main' into refactor/rust-structure-docs([13137cb](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/13137cbed9640ee7fe9d345dbc9c45f61b9c1694))

## [2026.8.18](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.17...2026.8.18) - 2026-08-15



### Bug Fixes

- let personality refine sound natural instead of stamping a cue([c37156e](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/c37156e95706d672921fba0d655b95f22389af91))

## [2026.8.17](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.16...2026.8.17) - 2026-08-15



### Bug Fixes

- include LICENSE and third-party notices in release artifacts([eaf0d32](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/eaf0d3203a2e7e4932e230052690445e739e721e))

## [2026.8.16](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.15...2026.8.16) - 2026-08-15



### Bug Fixes

- unify room light targeting and harden the parse path([ff33d9b](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/ff33d9bedad485888dd9570a382a479aa715d47f))

## [2026.8.15](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.14...2026.8.15) - 2026-08-15



### Bug Fixes

- harden API auth and infer action from the resolved target([79dfa13](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/79dfa1331edefd147965cc092edce671f82cf2b5))


### Other

- Merge pull request #58 from FABBricate-IT-Solutions/fix/auth-action-target-overlay([09eead6](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/09eead67210fb3082e0de27d6fdfd3aa4c84c0e6))


### Testing

- cover schalte das Wohnzimmerlicht an([fd455ea](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/fd455ea5acb3a5af2b59a6a73f91fdf263b0ea6d))

## [2026.8.14](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.13...2026.8.14) - 2026-08-15



### Bug Fixes

- speak HA display names and keep compound light status([517bb37](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/517bb3784f528610f52ee9a9a1b06d68b19e1f16))


### Other

- Merge pull request #56 from FABBricate-IT-Solutions/fix/addon-entity-display-names([f4d0080](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/f4d0080392145a4d3a18220174266e26dc73dd0b))

## [2026.8.13](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.12...2026.8.13) - 2026-08-15



### Features

- refine NLU replies in each personality voice([1380e2e](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/1380e2e6306dddcda5306fdcdd3b5d6d19c7f5c6))


### Other

- Merge pull request #54 from FABBricate-IT-Solutions/fix/climate-speech-refine-voice([f98ee19](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/f98ee19596636635e5dca16775f9320edf02ce1c))


### Styling

- rustfmt climate speech tests([8dec09b](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/8dec09b6ffcb29540bdbca78237be4f6a30d7663))

## [2026.8.12](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.11...2026.8.12) - 2026-08-15



### Bug Fixes

- detect formal news follow-up prompts([af6dcde](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/af6dcde6e9fd3e94c92ebbe5aa51b25d1c6439ca))


### Other

- Merge pull request #52 from FABBricate-IT-Solutions/fix/news-nudge-formal([f38c279](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/f38c279d17936ca0fe4bb834f7db297b4bbfa360))

## [2026.8.11](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.10...2026.8.11) - 2026-08-15



### Features

- refine NLU replies through the fallback LLM([5ec1b89](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/5ec1b89c5665426234b0c25d6ebf28ec5e8737f3))


### Other

- Merge pull request #50 from FABBricate-IT-Solutions/feat/llm-speech-refine([22f1232](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/22f12325e470862f13b20acc2299e9083683d99b))

## [2026.8.10](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.9...2026.8.10) - 2026-08-15



### Features

- route news questions through a briefing then the LLM([dec7875](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/dec78757648e80a6f1830e2786cc7603c7818576))


### Other

- Merge pull request #48 from FABBricate-IT-Solutions/feat/news-briefing([284689b](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/284689b3d2f9686c6a897112c89c30a535457973))

## [2026.8.9](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.8...2026.8.9) - 2026-08-15



### Bug Fixes

- elide needless lifetime on fallback_climate([b934148](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/b9341485a9aa8e483bcffbe11e6aee6ff0f766d1))


### Miscellaneous

- include ENGINE_VERSION in the release land commit([6c7bc7d](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/6c7bc7dd1578baa9b58bc54bdd70d66543e3fec8))


### Other

- Merge pull request #47 from FABBricate-IT-Solutions/refactor/packs-and-home-policy([0ca5ff0](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/0ca5ff0ac60c7542813d6784642e2113100a8e0a))


### Refactor

- extract language packs and home policy from the parse pipeline([483d93e](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/commit/483d93e9379da6e1ff5c33afefe1b1c1c7d3838b))

## [2026.8.8](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.7...2026.8.8) - 2026-08-15

### Bug Fixes

- Pick the only matching Heizung or Klimaanlage when the sentence has no room and no alias

## [2026.8.7](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.6...2026.8.7) - 2026-08-15

### Features

- Route casual and special speech to the LLM fallback even when that agent advertises Assist tools
- Distinguish Heizung and Klimaanlage by name, alias, and HA tags

### Bug Fixes

- Treat genitive room status (*Status der Küche*) as the area, not the kitchen lamp
- Set *Klimaanlage auf 20° / 20 Grad* on the AC instead of the bedroom heater

## [2026.8.6](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/compare/2026.8.5...2026.8.6) - 2026-08-15

### Bug Fixes

- Start the GHCR add-on image through `klar-entry.sh` so HTTP and Wyoming bind `0.0.0.0` even when Supervisor passes extra args
- Allow parse and GET from the Supervisor network without a token so Assist reaches the add-on out of the box

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
