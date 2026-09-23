# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---
## [0.23.1](https://github.com/DevelAngel/keep-it-done/compare/v0.23.0..v0.23.1) - 2026-09-23

### Bug Fixes

- switch start and due dates - ([02efb5d](https://github.com/DevelAngel/keep-it-done/commit/02efb5d16951e61f3531a54a7b66ad0adac2f0de)) - Angelos Drossos
- move add task field to top - ([1ae851c](https://github.com/DevelAngel/keep-it-done/commit/1ae851cdf48bda9962ce5464d0f89efaa0441aea)) - Angelos Drossos

---
## [0.23.0](https://github.com/DevelAngel/keep-it-done/compare/v0.22.0..v0.23.0) - 2026-08-21

### Bug Fixes

- **(app)** stop assignee input losing focus per keystroke - ([ab2787a](https://github.com/DevelAngel/keep-it-done/commit/ab2787ae894671536ccf2193569943e411e9e1fc)) - Angelos Drossos
- **(app)** stop contexts block remounting on every toggle - ([78d9760](https://github.com/DevelAngel/keep-it-done/commit/78d9760bedf966d9272de49b86fd41cc934424b4)) - Angelos Drossos
- **(mcp)** always list unassigned report variant - ([dc9f8e7](https://github.com/DevelAngel/keep-it-done/commit/dc9f8e7b7ac0dbf935a3df8987dbf700aab72581)) - Angelos Drossos
- **(server)** [**breaking**] timeout issues in report due to sampling requests - ([52f3b49](https://github.com/DevelAngel/keep-it-done/commit/52f3b492472f32f31d776f26b8bb7261a1e88480)) - Angelos Drossos
- **(types)** detect legacy '@'-prefixed contexts on load - ([db4c706](https://github.com/DevelAngel/keep-it-done/commit/db4c706ff463872c7e4359d8fde1cccbf02cda4c)) - Angelos Drossos
- **(types)** reserve 'unassigned' as assignee name - ([52599e7](https://github.com/DevelAngel/keep-it-done/commit/52599e70f770b2789fc88443e2b61c0ab576cda5)) - Angelos Drossos

### Documentation

- replace CLI with MCP server description - ([f567845](https://github.com/DevelAngel/keep-it-done/commit/f56784567749607d0a3105f1bb8e318e5095c67b)) - Angelos Drossos
- reframe README around value proposition - ([bb1533d](https://github.com/DevelAngel/keep-it-done/commit/bb1533ddf05c3af2171d6ed889bdd200dd9dbb46)) - Angelos Drossos

### Features

- **(app)** add assignee to task edit mode - ([5e80c35](https://github.com/DevelAngel/keep-it-done/commit/5e80c35b49e6b872f0c4e1240a6adc68930f61c9)) - Angelos Drossos
- **(app)** add Unassigned filter chip - ([d4341d6](https://github.com/DevelAngel/keep-it-done/commit/d4341d67db2cc24464f2df7eb981e631dbd17b80)) - Angelos Drossos
- **(filter)** make assignee filter single-select - ([3f4959e](https://github.com/DevelAngel/keep-it-done/commit/3f4959e369e0fcac7ad94f53e22689e79f3f0f4b)) - Angelos Drossos
- **(mcp)** add set_assignee tool - ([d6122e2](https://github.com/DevelAngel/keep-it-done/commit/d6122e2f8ce93f083f9a2c2e007eb7706a8a2d37)) - Angelos Drossos
- **(mcp)** add tool annotations to all tools - ([864b2d8](https://github.com/DevelAngel/keep-it-done/commit/864b2d80823799b92222642b9472bc0d6531f797)) - Angelos Drossos
- **(mcp)** add per-assignee daily report resource - ([b1e525a](https://github.com/DevelAngel/keep-it-done/commit/b1e525a35be3cad836e702bbc66effb21de47da6)) - Angelos Drossos
- **(mcp)** add per-assignee backlog, quick wins, weekly - ([b7f2532](https://github.com/DevelAngel/keep-it-done/commit/b7f2532bce5e7b6cb2a8897eb69a3f885083820d)) - Angelos Drossos
- **(mcp)** add unassigned-tasks report variant - ([3451a64](https://github.com/DevelAngel/keep-it-done/commit/3451a6473c27a7393434cb184de66adf4ccb1bcc)) - Angelos Drossos
- **(types)** [**breaking**] switch context prefix from '@' to '#' - ([4821c34](https://github.com/DevelAngel/keep-it-done/commit/4821c346718a47400d90739ee547e3a57df03aef)) - Angelos Drossos
- **(types)** add optional Assignee field to Infos - ([e95dd90](https://github.com/DevelAngel/keep-it-done/commit/e95dd90f24f9fcf8cd2e492cbd069f68dafa612f)) - Angelos Drossos
- **(web)** assignee filter chips and server fns - ([57c7626](https://github.com/DevelAngel/keep-it-done/commit/57c7626fa5290763f7ba07cd3a5b808b9ab9283d)) - Angelos Drossos

### Refactoring

- **(app)** derive_more Deref, readable format string - ([30a30a6](https://github.com/DevelAngel/keep-it-done/commit/30a30a6a8474819f2156862fef18af7f3a1383d2)) - Angelos Drossos
- **(mcp)** group non-filtered arms before filtered - ([56f3b5f](https://github.com/DevelAngel/keep-it-done/commit/56f3b5fffd450cbb0af1375f496984a61b216188)) - Angelos Drossos
- **(mcp)** move slug parsing/resources onto AssigneeFilter - ([9917264](https://github.com/DevelAngel/keep-it-done/commit/9917264d258bde5aae377ba56a4eae80056a5c57)) - Angelos Drossos
- **(types)** use derive_more for wrapper types - ([7a9a2d5](https://github.com/DevelAngel/keep-it-done/commit/7a9a2d5119365d2ea94fdcc48ad5d8d90de31cfa)) - Angelos Drossos

### Style

- **(mcp)** idiomatic slug parsing and signature wrap - ([ba30590](https://github.com/DevelAngel/keep-it-done/commit/ba30590054ba2502fccc9a497f7dd96929ee2e1d)) - Angelos Drossos

---
## [0.22.0](https://github.com/DevelAngel/keep-it-done/compare/v0.21.0..v0.22.0) - 2026-08-01

### Bug Fixes

- **(server)** oauth server issuer mismatch - ([d3e9024](https://github.com/DevelAngel/keep-it-done/commit/d3e90247c27eb846e890b6f7f6cd83d667b71d84)) - Angelos Drossos

### Build

- **(server)** bump rmcp from 2.2.0 to 3.0.1 - ([7a40e75](https://github.com/DevelAngel/keep-it-done/commit/7a40e751a3787dbff09df5101143939289f670a3)) - Angelos Drossos

### Features

- **(server)** use MCP sampling for better reports - ([2716b32](https://github.com/DevelAngel/keep-it-done/commit/2716b32f9ccd5ffab066ee1a6a71f47ffc8d81a7)) - Angelos Drossos
- **(server)** add weekly report of recent changes - ([09ad0ae](https://github.com/DevelAngel/keep-it-done/commit/09ad0ae299228e0e91d10ed878474ed586ab4c7b)) - Angelos Drossos

---
## [0.21.0](https://github.com/DevelAngel/keep-it-done/compare/v0.20.0..v0.21.0) - 2026-07-31

### Bug Fixes

- **(e2e)** replace tarpc harness with mcp + admin channel - ([d4418d0](https://github.com/DevelAngel/keep-it-done/commit/d4418d065641c5f4c54026e25a5038b5d095de76)) - Angelos Drossos
- **(server)** stop leaking secrets via logs/Debug - ([154380b](https://github.com/DevelAngel/keep-it-done/commit/154380b31d1920585de952ba5b4d9ee2785a1c29)) - Angelos Drossos

### Build

- **(server)** add rmcp crate - ([d179adf](https://github.com/DevelAngel/keep-it-done/commit/d179adfd85f8ef1be13616f0015cc2d375b05cad)) - Angelos Drossos
-  [**breaking**]remove kid-cli - ([34f2a30](https://github.com/DevelAngel/keep-it-done/commit/34f2a3039f5397e012198814c325dbbdfa42a1d9)) - Angelos Drossos

### Documentation

- add MCP Server ADR - ([6c1ab75](https://github.com/DevelAngel/keep-it-done/commit/6c1ab7535d41cc238d31d9d7b8a9673ffeb52d47)) - Angelos Drossos
- remove CLI SKILL - ([6da0b5f](https://github.com/DevelAngel/keep-it-done/commit/6da0b5f2af504d286fad8016124bb91e6b9e9469)) - Angelos Drossos

### Features

- **(server)** [**breaking**] replace rpc with mcp service - ([145b7c8](https://github.com/DevelAngel/keep-it-done/commit/145b7c8965c76834d66c778672387170f61af685)) - Angelos Drossos
- **(server)** [**breaking**] add OAuth 2.1 authorization to MCP server - ([c16fb76](https://github.com/DevelAngel/keep-it-done/commit/c16fb7664097cc61d509f6b0c6e127bc4a92e5ac)) - Angelos Drossos
- **(server)** support client_credentials grant for machine MCP clients - ([80c6f60](https://github.com/DevelAngel/keep-it-done/commit/80c6f60ad668e2382c5ec64554cb0d14430e9389)) - Angelos Drossos
- **(server)** expose daily_report and backlog as MCP resources - ([1f891a6](https://github.com/DevelAngel/keep-it-done/commit/1f891a65638bea86b42526bfbec4e05dd21dd83a)) - Angelos Drossos
- **(server)** derive actor from the authenticated OAuth client, config-driven - ([6727879](https://github.com/DevelAngel/keep-it-done/commit/672787978a50e52dc985640ef42c3d003aa6bb9d)) - Angelos Drossos
- **(server)** expose quick wins as an MCP resource - ([0d620cd](https://github.com/DevelAngel/keep-it-done/commit/0d620cd8bae386bb7e684590958e60c26c6608dc)) - Angelos Drossos
- **(server)** serve MCP favicons via LEPTOS_SITE_ROOT - ([cd7f7bf](https://github.com/DevelAngel/keep-it-done/commit/cd7f7bfd2e7ea4d039d2f79f6d7a15475ea5e52c)) - Angelos Drossos
- **(server)** give the daily reports a Zelda-flavored voice - ([a3fefaf](https://github.com/DevelAngel/keep-it-done/commit/a3fefaf35d9d2be0a1741531fecfc2fc1f209384)) - Angelos Drossos
- **(types)** [**breaking**] add schemars derive for mcp - ([22d59c5](https://github.com/DevelAngel/keep-it-done/commit/22d59c52775667a00020478cc942d30b01c97023)) - Angelos Drossos

### Miscellaneous Chores

- **(app)** fix formatting - ([eb108d3](https://github.com/DevelAngel/keep-it-done/commit/eb108d3a4229921b637c67c86e0d5a7c30a7840e)) - Angelos Drossos
- **(server)** remove warnings - ([563ebf7](https://github.com/DevelAngel/keep-it-done/commit/563ebf7fe5c3dbb66af6a1339da716357960fc79)) - Angelos Drossos
- formatting fixes - ([07aa5f4](https://github.com/DevelAngel/keep-it-done/commit/07aa5f448d6d622dc2cb1406cd35ef1df56ab4b9)) - Angelos Drossos
- set verbosity to info level by default - ([0065190](https://github.com/DevelAngel/keep-it-done/commit/006519022e998eafb26cd85c79a5075910b1efd2)) - Angelos Drossos

### Refactoring

- **(server)** split oauth.rs into module - ([7902754](https://github.com/DevelAngel/keep-it-done/commit/7902754c19e37f1e5d7ed23c63a20e943a5d02ce)) - Angelos Drossos

---
## [0.20.0](https://github.com/DevelAngel/keep-it-done/compare/v0.19.0..v0.20.0) - 2026-07-10

### Documentation

- **(adr)** add PWA decision record - ([a0e577f](https://github.com/DevelAngel/keep-it-done/commit/a0e577fa6426a9ee91cedb844493c1769d392289)) - Angelos Drossos

### Features

- **(app)** add PWA icon set (192x192, 512x512) - ([cbf1781](https://github.com/DevelAngel/keep-it-done/commit/cbf178159cd8332f3f4bf04966433aede95b7bd7)) - Angelos Drossos
- **(app)** add web app manifest and PWA meta tags - ([bbd7547](https://github.com/DevelAngel/keep-it-done/commit/bbd75475524debc09be8d43736932bb76e28098d)) - Angelos Drossos
- **(app)** add stub service worker for PWA installability - ([f6ca264](https://github.com/DevelAngel/keep-it-done/commit/f6ca264482017107437d6f988e35d8b9c6b90c1b)) - Angelos Drossos

### Miscellaneous Chores

- rename CLAUDE.md to AGENTS.md - ([e1b2be2](https://github.com/DevelAngel/keep-it-done/commit/e1b2be296f80ccb29e30fa067f0e708ef5c0f1cd)) - Angelos Drossos

---
## [0.19.0](https://github.com/DevelAngel/keep-it-done/compare/v0.18.0..v0.19.0) - 2026-05-26

### Documentation

- add ADR and UXDR for fuzzy search - ([969ce67](https://github.com/DevelAngel/keep-it-done/commit/969ce67155608010804a5877adf45fb168ffea71)) - Angelos Drossos

### Features

- **(app)** add client-side fuzzy search to All Open and What I Finished - ([810be3a](https://github.com/DevelAngel/keep-it-done/commit/810be3ae69e944c8365979d2f20efddcdd80332f)) - Angelos Drossos
- **(app)** add client-side fuzzy search to All Open and What I Finished - ([5c10bd9](https://github.com/DevelAngel/keep-it-done/commit/5c10bd90b37ebef6793482e6d1762bc230d333c1)) - Angelos Drossos
- **(cli)** overhaul list command - ([1cb90d7](https://github.com/DevelAngel/keep-it-done/commit/1cb90d719062a316c539a4b7d37780a7c5ed5f7a)) - Angelos Drossos

### Tests

- **(app)** add unit tests for apply_search fuzzy matching - ([4770f32](https://github.com/DevelAngel/keep-it-done/commit/4770f32bfc181bde16b373fef4440fc0af0c0197)) - Angelos Drossos

---
## [0.18.0](https://github.com/DevelAngel/keep-it-done/compare/v0.17.2..v0.18.0) - 2026-05-25

### Bug Fixes

- **(app)** hide delete button outside edit mode - ([037f826](https://github.com/DevelAngel/keep-it-done/commit/037f82621ee5d9489302640f968d94cfa5bf7c0b)) - Angelos Drossos
- **(app)** integrate delete button into task timeline - ([a423626](https://github.com/DevelAngel/keep-it-done/commit/a423626bbbff77724d81c35e5a0eb819f99c033d)) - Angelos Drossos
- **(e2e)** create real dirty state for flush LED e2e tests - ([a8239aa](https://github.com/DevelAngel/keep-it-done/commit/a8239aa923bbf29cc3231d6c111783f28e6ce592)) - Angelos Drossos
- **(e2e)** activate edit mode before delete - ([fcbcf1d](https://github.com/DevelAngel/keep-it-done/commit/fcbcf1d255156d83b40ccec45736ebc98bbd84a5)) - Angelos Drossos

### Documentation

- add UXDR for task deletion UI - ([942c5e1](https://github.com/DevelAngel/keep-it-done/commit/942c5e14d1805f8f6d40858e1d35b737def766f3)) - Angelos Drossos

### Features

- **(app)** add inline-confirm delete button to task detail - ([db87103](https://github.com/DevelAngel/keep-it-done/commit/db87103e3be58a419a1ca6580b967876618a795f)) - Angelos Drossos
- **(app)** add flush status LED via SSE push channel - ([95fd02a](https://github.com/DevelAngel/keep-it-done/commit/95fd02a8bfc1427a508901cbd534c46dfc845c85)) - Angelos Drossos

### Tests

- **(e2e)** add tests for task deletion - ([b44b252](https://github.com/DevelAngel/keep-it-done/commit/b44b252d531b276c7a7eeb8c8824e24c68ad6527)) - Angelos Drossos
- **(e2e)** add tests for flush status LED - ([e8e7e11](https://github.com/DevelAngel/keep-it-done/commit/e8e7e11222fd8dfe8fd4e913f580798a2b51a05d)) - Angelos Drossos

---
## [0.17.2](https://github.com/DevelAngel/keep-it-done/compare/v0.17.1..v0.17.2) - 2026-05-24

### Bug Fixes

- **(app)** use English for error template messages - ([50d6bc5](https://github.com/DevelAngel/keep-it-done/commit/50d6bc5e47bdd7aba6af45d16c68d7e6d3953140)) - Angelos Drossos
- **(app)** remove reload link from session-expired - ([f0fbb10](https://github.com/DevelAngel/keep-it-done/commit/f0fbb1060dc09b45d2487957d2a4f4aa54438263)) - Angelos Drossos

### Tests

- **(e2e)** verify session-expired message - ([1225bcb](https://github.com/DevelAngel/keep-it-done/commit/1225bcb64228b2023753b22829e908b5f7e2ecd4)) - Angelos Drossos

---
## [0.17.1](https://github.com/DevelAngel/keep-it-done/compare/v0.17.0..v0.17.1) - 2026-05-23

### Bug Fixes

- **(app)** show friendly message on expired session - ([7308471](https://github.com/DevelAngel/keep-it-done/commit/73084712a2b2fb2f13744128015fae69e460f3d7)) - Angelos Drossos

### Documentation

- add E2E time simulation concept - ([b58f97a](https://github.com/DevelAngel/keep-it-done/commit/b58f97abc1456d92f191faa47c0a0b8eb5e9f449)) - Angelos Drossos
- add ADR for central time accessor - ([2743d5f](https://github.com/DevelAngel/keep-it-done/commit/2743d5ffec7bc3e4bec16bb14ae5bbe6ba8664c0)) - Angelos Drossos

### Refactoring

- add central time accessor with offset - ([5cc1883](https://github.com/DevelAngel/keep-it-done/commit/5cc1883561223aab46dfbe0569cdb8a766d9fc08)) - Angelos Drossos

### Tests

- **(e2e)** add time simulation scenarios - ([5703470](https://github.com/DevelAngel/keep-it-done/commit/5703470a1303c6e89e104de07c3ba5673ee02e38)) - Angelos Drossos
- **(e2e)** use time simulation in screenshot feature - ([a504f78](https://github.com/DevelAngel/keep-it-done/commit/a504f788531e982cde2ce379fba383f8797d0179)) - Angelos Drossos

---
## [0.17.0](https://github.com/DevelAngel/keep-it-done/compare/v0.16.0..v0.17.0) - 2026-05-23

### Documentation

- add urgency-checkbox UXDR for Upcoming - ([69ad153](https://github.com/DevelAngel/keep-it-done/commit/69ad153c310aa0b5f453fa3b8724a4e74ec032bf)) - Angelos Drossos

### Features

- **(app)** urgency accent on Upcoming checkbox - ([f375cc2](https://github.com/DevelAngel/keep-it-done/commit/f375cc2037f4ba53360dcadbb351c6fb11b93e33)) - Angelos Drossos

### Tests

- **(app)** urgency assertions for group_upcoming - ([2187c98](https://github.com/DevelAngel/keep-it-done/commit/2187c98d5ff9f67db4fc5f5c24618d30545441d7)) - Angelos Drossos
- **(e2e)** replace attention-label with urgency steps - ([dee46e6](https://github.com/DevelAngel/keep-it-done/commit/dee46e6a5838233268ff1835ef80328d9b31aede)) - Angelos Drossos

---
## [0.16.0](https://github.com/DevelAngel/keep-it-done/compare/v0.13.1..v0.16.0) - 2026-05-21

### Bug Fixes

- **(app)** switch to All Open on Add Task tap - ([004b7c2](https://github.com/DevelAngel/keep-it-done/commit/004b7c26ec0879442232698f8f11dd537b2b8cf0)) - Angelos Drossos
- **(app)** scroll to Add Task input after view switch - ([425cef5](https://github.com/DevelAngel/keep-it-done/commit/425cef50f69327e3884918fb421db89d40fa3996)) - Angelos Drossos
- **(app)** respect availability in start_date override - ([0f00407](https://github.com/DevelAngel/keep-it-done/commit/0f0040715acb51e15a1f26dffd6c6de01d5f1b5d)) - Angelos Drossos
- **(types)** drop 5-min debounce from add_author - ([1edb016](https://github.com/DevelAngel/keep-it-done/commit/1edb01646bc1fca338bc6a6ba1db0a80355e532a)) - Angelos Drossos

### Documentation

- explicit sort chain for Ready to Start - ([9702f3e](https://github.com/DevelAngel/keep-it-done/commit/9702f3e9c57a50817d08b9be5c5141cb51ba2ed3)) - Angelos Drossos
- add author tracking concept - ([829c0cc](https://github.com/DevelAngel/keep-it-done/commit/829c0cc15312374a45ea3a7ed62e448eabe91dde)) - Angelos Drossos
- add UXDR for Add Task view switch - ([10b0e83](https://github.com/DevelAngel/keep-it-done/commit/10b0e8375667b0d2646afb6f0355c451f30392c1)) - Angelos Drossos

### Features

- **(app)** sort Quick Wins by priority within groups - ([3dd8780](https://github.com/DevelAngel/keep-it-done/commit/3dd8780dd39ee48aeaba647e6d871e2bc5bed73a)) - Angelos Drossos
- **(app)** switch to All Open on Add Task tap - ([93e894a](https://github.com/DevelAngel/keep-it-done/commit/93e894a2b3a683dfac13a94970d8053e5f548a94)) - Angelos Drossos
- **(app)** add Tomorrow deadline group - ([43cb1f6](https://github.com/DevelAngel/keep-it-done/commit/43cb1f6535dd90ea12197d3bb6cb3a6633c7c156)) - Angelos Drossos
- **(app)** add weekend separator in ThisWeek/NextWeek - ([9247d57](https://github.com/DevelAngel/keep-it-done/commit/9247d572efbd4768262c80b27a69eea0b932c3b2)) - Angelos Drossos
- **(app)** dynamic weekend label for upcoming groups - ([b67d5ae](https://github.com/DevelAngel/keep-it-done/commit/b67d5ae65e9a69915ee00b3cac69273b4da0613e)) - Angelos Drossos
- **(app)** show category tag below summary in non-category views - ([270f73a](https://github.com/DevelAngel/keep-it-done/commit/270f73ae745666a53494186203863e64b0b543ca)) - Angelos Drossos
- **(app)** always wrap attention label into two lines - ([fe27a5b](https://github.com/DevelAngel/keep-it-done/commit/fe27a5b436bf477fb308e1f3ccd2cab199d05b0a)) - Angelos Drossos
- **(app)** show contexts in category-grouped views - ([d0bced0](https://github.com/DevelAngel/keep-it-done/commit/d0bced000f1fee155bb6396b9663ac81ae33f0c6)) - Angelos Drossos

### Miscellaneous Chores

- **(e2e)** order e2e features by complexity - ([961e838](https://github.com/DevelAngel/keep-it-done/commit/961e838030d12522ffc179d87f27de757b6c229b)) - Angelos Drossos
- **(goose)** add just recipes to .goosehints - ([dc716b9](https://github.com/DevelAngel/keep-it-done/commit/dc716b9fa88b3ae56f25946de556b12187bc8b05)) - Angelos Drossos
- **(just)** kill server before e2e test - ([4b80885](https://github.com/DevelAngel/keep-it-done/commit/4b80885e03a446538acc312b87c7c602fa0d5f3f)) - Angelos Drossos
- update screenshots - ([7f95d36](https://github.com/DevelAngel/keep-it-done/commit/7f95d3677f8ade59d567ce572bf90df7fde307a3)) - Angelos Drossos
- add justfile - ([f943a0e](https://github.com/DevelAngel/keep-it-done/commit/f943a0e5fcd7279cbecff9cee042a0d0134282fd)) - Angelos Drossos

### Refactoring

- **(app)** sort What I Finished per group - ([dee1d9f](https://github.com/DevelAngel/keep-it-done/commit/dee1d9f1922cc6415a79da2fe1d1995b4f929f18)) - Angelos Drossos
- **(e2e)** seed tasks as JSON files - ([30aa443](https://github.com/DevelAngel/keep-it-done/commit/30aa4433b55b6b94a646f82d2ea5ab901fc813b2)) - Angelos Drossos

### Tests

- **(app)** verify intra-group sort for Quick Wins and What I Finished - ([a217140](https://github.com/DevelAngel/keep-it-done/commit/a217140bfd4acb7d7157c1fbcb2d5a1b07f28e2d)) - Angelos Drossos
- **(app)** add All Open and Recently Changed tests - ([d381615](https://github.com/DevelAngel/keep-it-done/commit/d38161529bab429c07d6713fe796a5af24c94f1f)) - Angelos Drossos
- **(app)** start_date override ignores availability - ([06db6f0](https://github.com/DevelAngel/keep-it-done/commit/06db6f08a41d3096801e459229307a0abcaf8066)) - Angelos Drossos
- **(app)** add Tomorrow deadline group tests - ([ed59aa7](https://github.com/DevelAngel/keep-it-done/commit/ed59aa7d6573673cdea65afc921125a4ecef70dc)) - Angelos Drossos
- **(app)** display_label weekend-aware labels - ([00fa43f](https://github.com/DevelAngel/keep-it-done/commit/00fa43fb6bae85ec04877c5e3b96cf880bb7b5cd)) - Angelos Drossos
- **(e2e)** add task view switch and auth - ([9fc2c72](https://github.com/DevelAngel/keep-it-done/commit/9fc2c7280c1e31c03f16444804798e0aaff36faf)) - Angelos Drossos
- **(e2e)** e2e scroll to Add Task input - ([4893672](https://github.com/DevelAngel/keep-it-done/commit/48936726cfa648d308d62ef6cd2963c0ba1f16f7)) - Angelos Drossos
- **(types)** add_author scenarios - ([c62b084](https://github.com/DevelAngel/keep-it-done/commit/c62b08428566021420a34a4f77905e40c6023017)) - Angelos Drossos

---
## [0.13.1](https://github.com/DevelAngel/keep-it-done/compare/v0.12.0..v0.13.1) - 2026-05-11

### Documentation

- add test instructions - ([9e3f3ea](https://github.com/DevelAngel/keep-it-done/commit/9e3f3ea1c367e719650370fe57338343d79cf349)) - Angelos Drossos
- add Availability + Attention Date concept - ([529501a](https://github.com/DevelAngel/keep-it-done/commit/529501a33ed5a8023ad30894757da722db151c50)) - Angelos Drossos

### Features

- **(app)** support ?view= and ?expand= params - ([23ca5c7](https://github.com/DevelAngel/keep-it-done/commit/23ca5c798010851f734fc2f7914587cb733b3ddf)) - Angelos Drossos
- **(app)** collapsible backlog disclosure in Upcoming - ([b60a0e5](https://github.com/DevelAngel/keep-it-done/commit/b60a0e50a6c47635a34745db1f55ef0874fe8fc6)) - Angelos Drossos
- **(app)** group Upcoming by attention date - ([f1bdb4f](https://github.com/DevelAngel/keep-it-done/commit/f1bdb4f2f6e8961d9825b5524aa672f5145c0e6d)) - Angelos Drossos
- **(app)** show attention indicator in Upcoming view - ([07dc825](https://github.com/DevelAngel/keep-it-done/commit/07dc825b1a6e4f790ff9ba9430e224640a20bef0)) - Angelos Drossos
- **(app)** group by start_date when earlier than attention - ([4ee9296](https://github.com/DevelAngel/keep-it-done/commit/4ee9296d315eeb3a6c35f2bf01d0888acf4df285)) - Angelos Drossos
- **(rpc)** add count method - ([bf78638](https://github.com/DevelAngel/keep-it-done/commit/bf78638b28b0a46db155aabf3f4379355a39bae9)) - Angelos Drossos
- **(rpc)** add switch_dir to change task directory - ([bb87914](https://github.com/DevelAngel/keep-it-done/commit/bb879145be05f2dd40e66e1dba1cc120f2d9edc0)) - Angelos Drossos
- **(rpc)** add add_with_id for custom UUID v7 - ([6226246](https://github.com/DevelAngel/keep-it-done/commit/6226246c16dcfeef8e58075a3ce784e45f45c8be)) - Angelos Drossos
- **(server)** add --tasks-dir option - ([e78e466](https://github.com/DevelAngel/keep-it-done/commit/e78e466231ac06300d6796f69b9e391b097d00aa)) - Angelos Drossos
- **(types)** [**breaking**] use CWD as default task directory - ([cb80ce3](https://github.com/DevelAngel/keep-it-done/commit/cb80ce356701df9a74e7e215afe497f9c670a42d)) - Angelos Drossos
- **(types)** add FromStr for TimeEstimate - ([9f5bd54](https://github.com/DevelAngel/keep-it-done/commit/9f5bd54150ec23a043420a6d106a3c8396dba91e)) - Angelos Drossos
- **(types)** add Availability scheduling constraint - ([ce1a63d](https://github.com/DevelAngel/keep-it-done/commit/ce1a63d0168148d89bc92d691cc7bd0d78b2aa7e)) - Angelos Drossos
- **(types)** add TimeEstimate::lead_days() - ([9bcf3ba](https://github.com/DevelAngel/keep-it-done/commit/9bcf3baea76b6975bfc6b7a016528172a862840b)) - Angelos Drossos
- **(types)** add attention_date() computation - ([4d033a6](https://github.com/DevelAngel/keep-it-done/commit/4d033a63be7338b72ac8f4fdef5976c939557e6d)) - Angelos Drossos
- add availability toggle to task detail UI - ([53b8b5f](https://github.com/DevelAngel/keep-it-done/commit/53b8b5f3e8bc2056e869354adbc1baa03a60e830)) - Angelos Drossos

### Miscellaneous Chores

- update screenshots - ([d8c06ba](https://github.com/DevelAngel/keep-it-done/commit/d8c06bae6b6eb7693da0e48f74eed7c26915c876)) - Angelos Drossos
- update screenshots - ([8cd697a](https://github.com/DevelAngel/keep-it-done/commit/8cd697a995303675179f21e22b07aee5676b161a)) - Angelos Drossos

### Refactoring

- **(app)** extract internal SSR helpers into submodule - ([3123a3b](https://github.com/DevelAngel/keep-it-done/commit/3123a3b5d642e9c7fc518c6a59f856f1023ebac7)) - Angelos Drossos
- **(cli)** split into lib and binary - ([b470439](https://github.com/DevelAngel/keep-it-done/commit/b470439e95077dcc4c5968728e76755d09c5de01)) - Angelos Drossos
- **(e2e)** split monolith into modules - ([4438a17](https://github.com/DevelAngel/keep-it-done/commit/4438a178b2ded3885ea6b1c3dc9996c538c90cf8)) - Angelos Drossos
- **(e2e)** consolidate task seeds into data table - ([e957cd6](https://github.com/DevelAngel/keep-it-done/commit/e957cd66f450a903a5a7cd8310acd2c1e92eedf3)) - Angelos Drossos
- extract view URL slugs into kid-types - ([0a7e7b4](https://github.com/DevelAngel/keep-it-done/commit/0a7e7b455594bd84de969945336bb64cebb80f9e)) - Angelos Drossos

### Tests

- **(app)** comprehensive deadline_group coverage - ([cf3fb15](https://github.com/DevelAngel/keep-it-done/commit/cf3fb15456f18ebc3526ba8d34cd9aefd2f2288f)) - Angelos Drossos
- **(app)** unit tests for Upcoming view grouping - ([a16ab7d](https://github.com/DevelAngel/keep-it-done/commit/a16ab7d152be0c56afaf6e9946266293ea44402c)) - Angelos Drossos
- **(app)** weekday-only and availability+start_date integration coverage - ([ab43b17](https://github.com/DevelAngel/keep-it-done/commit/ab43b17e0e1b0c2d9f69b9aa3f54acf7cf4977e4)) - Angelos Drossos
- **(e2e)** use cucumber for end2end testing - ([afc4eed](https://github.com/DevelAngel/keep-it-done/commit/afc4eed13a4ac55957ba834380d4838228851907)) - Angelos Drossos
- **(e2e)** use thirtyfour for end2end testing - ([2069380](https://github.com/DevelAngel/keep-it-done/commit/2069380285e3e953ce6e7f6ae3124b571a742e43)) - Angelos Drossos
- **(e2e)** combine cucumber with thirtyfour - ([89f7315](https://github.com/DevelAngel/keep-it-done/commit/89f7315933a5edabda37743da11c970cf9dc9899)) - Angelos Drossos
- **(e2e)** isolate task data per scenario - ([f6b7857](https://github.com/DevelAngel/keep-it-done/commit/f6b78576592db723112ef13920a25c5f37c61ec6)) - Angelos Drossos
- **(e2e)** cleanup example and wiki - ([586ed1b](https://github.com/DevelAngel/keep-it-done/commit/586ed1b357d42073fb99bc2574412c5d9728d7ee)) - Angelos Drossos
- **(e2e)** auto-update README screenshots - ([8852734](https://github.com/DevelAngel/keep-it-done/commit/8852734c700dc0c306f4d173fcd3611770949dd8)) - Angelos Drossos
- **(e2e)** seed tasks with individual Given steps - ([01434e3](https://github.com/DevelAngel/keep-it-done/commit/01434e3da6c51fb8f520bb0e60b1d2dc0b2d44ef)) - Angelos Drossos
- **(e2e)** verify task list renders on each view - ([aa35aab](https://github.com/DevelAngel/keep-it-done/commit/aa35aab01f29c0b044400db4713644c18fcec0b2)) - Angelos Drossos
- **(e2e)** separate screenshots from nav test - ([5d28275](https://github.com/DevelAngel/keep-it-done/commit/5d28275b5b3793da66c16c91246788f2ac30f0d3)) - Angelos Drossos
- **(e2e)** add start/due dates to screenshot seeds - ([3fa7381](https://github.com/DevelAngel/keep-it-done/commit/3fa7381b4a791fb8daba22dc3813d98819b64922)) - Angelos Drossos
- **(e2e)** increase viewport height for screenshots - ([5dd32a1](https://github.com/DevelAngel/keep-it-done/commit/5dd32a172bdfefbc5e0dab05293f4404836b04d6)) - Angelos Drossos
- **(e2e)** add task-detail-expansion screenshot - ([d8fe73f](https://github.com/DevelAngel/keep-it-done/commit/d8fe73f588488c2a95d844476fe3401c8df72cca)) - Angelos Drossos
- **(e2e)** add Upcoming view step definitions - ([e3798bb](https://github.com/DevelAngel/keep-it-done/commit/e3798bb61e26144fdb1204a322c88f07101d64b2)) - Angelos Drossos
- **(types)** Availability serde and round-trip tests - ([44dd2b2](https://github.com/DevelAngel/keep-it-done/commit/44dd2b26714c9771bda0afdc8f92be11c450f6d5)) - Angelos Drossos
- **(types)** lead_days exhaustive coverage - ([ccd9560](https://github.com/DevelAngel/keep-it-done/commit/ccd9560fb00b7fe0366b330e04dfaa03ba16933e)) - Angelos Drossos
- **(types)** attention_date and is_eligible coverage - ([5704caf](https://github.com/DevelAngel/keep-it-done/commit/5704cafdf4930eee9aa7f4def6f54c7b29e545aa)) - Angelos Drossos

---
## [0.12.0](https://github.com/DevelAngel/keep-it-done/compare/v0.11.2..v0.12.0) - 2026-05-05

### Documentation

- add UXDRs for Upcoming view and view order - ([52662c4](https://github.com/DevelAngel/keep-it-done/commit/52662c4ebbc12158102d62741b2ddf3bd81a707b)) - Angelos Drossos
- update screenshots - ([de01f04](https://github.com/DevelAngel/keep-it-done/commit/de01f04e90eae356c973c83e7fa9ebedae7566ee)) - Angelos Drossos

### Features

- **(app)** add DeadlineGroup enum and apply_filter arm - ([d8f503a](https://github.com/DevelAngel/keep-it-done/commit/d8f503ad1e6bb528e0a45ace5511f9debee5976b)) - Angelos Drossos
- **(app)** reorder views, rename MyDay to AllOpen, add Upcoming - ([177d8ec](https://github.com/DevelAngel/keep-it-done/commit/177d8ecb6decbbd9f87705a404af1e8f3d726c52)) - Angelos Drossos
- **(app)** add fetch_upcoming, rename fetch_my_day - ([ecbbbaf](https://github.com/DevelAngel/keep-it-done/commit/ecbbbaffbfba7679278590a8bd6c1bab2736dcdd)) - Angelos Drossos
- **(app)** render Upcoming view with deadline groups - ([eef0430](https://github.com/DevelAngel/keep-it-done/commit/eef0430da459f8aa04db38b89254b67dcbdab553)) - Angelos Drossos

### Refactoring

- **(app)** reorder views — Quick Wins to position 2 - ([93d7e84](https://github.com/DevelAngel/keep-it-done/commit/93d7e845550ea40c4dd72ae46555a24d414d42ee)) - Angelos Drossos

### Style

- **(app)** harmonize view color palette - ([df58944](https://github.com/DevelAngel/keep-it-done/commit/df58944cbdc29eff8317078ef7802169ea752897)) - Angelos Drossos

---
## [0.11.2](https://github.com/DevelAngel/keep-it-done/compare/v0.11.1..v0.11.2) - 2026-05-01

### Documentation

- add UXDR for Quick Wins estimate grouping - ([e2848e6](https://github.com/DevelAngel/keep-it-done/commit/e2848e644df65356c24d18120706dfaceff97807)) - Angelos Drossos

### Features

- **(app)** group Quick Wins by time estimate - ([cc8afd1](https://github.com/DevelAngel/keep-it-done/commit/cc8afd18050c872a678427b161cd8e925423f607)) - Angelos Drossos

---
## [0.11.1](https://github.com/DevelAngel/keep-it-done/compare/v0.11.0..v0.11.1) - 2026-05-01

### Bug Fixes

- **(app)** show load-more button in empty recent view - ([4080540](https://github.com/DevelAngel/keep-it-done/commit/4080540392670aca9b100da93aaf6261e682d8ef)) - Angelos Drossos
- **(app)** clean up recent-changes view UX - ([336ea31](https://github.com/DevelAngel/keep-it-done/commit/336ea31ba123ea7fcbde4ebeba88eb41876cb430)) - Angelos Drossos

---
## [0.11.0](https://github.com/DevelAngel/keep-it-done/compare/v0.10.0..v0.11.0) - 2026-05-01

### Documentation

- add ADR for patch strategy evolution - ([be7bb4f](https://github.com/DevelAngel/keep-it-done/commit/be7bb4f59d5446c82dfaafacd51b74b92894f550)) - Angelos Drossos
- add UXDR for priority visual weight - ([b207138](https://github.com/DevelAngel/keep-it-done/commit/b2071385edb34704356617f65dfdeb0adf0a58d2)) - Angelos Drossos

### Features

- **(app)** add priority accent border for A-tasks - ([3468f8c](https://github.com/DevelAngel/keep-it-done/commit/3468f8cf109926b72d69128c1bf343127b2bbc0c)) - Angelos Drossos
- **(cli)** add set-priority command - ([8910755](https://github.com/DevelAngel/keep-it-done/commit/891075557b240901186bba88852dcaae6b2818fb)) - Angelos Drossos

### Refactoring

- **(types)** [**breaking**] move priority from Details to Infos - ([456c499](https://github.com/DevelAngel/keep-it-done/commit/456c49994ccbaa755c01a75b0f066158584a9895)) - Angelos Drossos

---
## [0.10.0](https://github.com/DevelAngel/keep-it-done/compare/v0.9.2..v0.10.0) - 2026-05-01

### Bug Fixes

- **(app)** move contexts above details - ([a846e85](https://github.com/DevelAngel/keep-it-done/commit/a846e853dc4a56a8d15869e4cc00f42165ca2595)) - Angelos Drossos

### Build

- **(types)** add indexmap2 feature to schemars - ([8ef9acc](https://github.com/DevelAngel/keep-it-done/commit/8ef9acc7dc3dd11809f7bc5120e507e9e01718a4)) - Angelos Drossos

### Documentation

- **(ai)** add local LLM server concept - ([76809d5](https://github.com/DevelAngel/keep-it-done/commit/76809d5910d2fa7cbc12ab221383d46868581176)) - Angelos Drossos
- **(uxdr)** add recent changes view rationale - ([6552463](https://github.com/DevelAngel/keep-it-done/commit/6552463ef1f0f8a6cc9188dce2eda8f707e07032)) - Angelos Drossos

### Features

- **(app)** use touched in Recent Changes view - ([483de2c](https://github.com/DevelAngel/keep-it-done/commit/483de2cced687efd98a14c2209dcff334939b81a)) - Angelos Drossos
- **(app)** show authors in task detail timeline - ([7f4a9ed](https://github.com/DevelAngel/keep-it-done/commit/7f4a9edc8d05d64f748881c44e16137a0e5c0522)) - Angelos Drossos
- **(app)** show authors in task details for all views - ([0d745c0](https://github.com/DevelAngel/keep-it-done/commit/0d745c0735e5b834eace53fa74aa3e0cfaeff8a1)) - Angelos Drossos
- **(app)** group recent changes by day - ([5426982](https://github.com/DevelAngel/keep-it-done/commit/54269821a94a1a360d3857bd6dec49d55baefd70)) - Angelos Drossos
- **(app)** refetch recent changes at midnight - ([7700498](https://github.com/DevelAngel/keep-it-done/commit/7700498bcff5926397e11ce27d503eec7b51dd32)) - Angelos Drossos
- **(app)** load more days in recent changes - ([97f2a18](https://github.com/DevelAngel/keep-it-done/commit/97f2a18689d95744fca1209b1ac4ed242c351576)) - Angelos Drossos
- **(app)** improve display of ai assistant name - ([593d5f0](https://github.com/DevelAngel/keep-it-done/commit/593d5f04a21667b8d6a314a612514cd3bd1505b2)) - Angelos Drossos
- **(cli)** [**breaking**] require human initiator for actions - ([ae1ec93](https://github.com/DevelAngel/keep-it-done/commit/ae1ec9356eff7d37b4c7239e7551698f382dbf2b)) - Angelos Drossos
- **(server)** prefix AI authors with "ai:" namespace - ([cfa9c38](https://github.com/DevelAngel/keep-it-done/commit/cfa9c3838fee1cc41907b05bf67a42e883733513)) - Angelos Drossos
- **(types)** add touched timestamp to Task - ([3ec6a09](https://github.com/DevelAngel/keep-it-done/commit/3ec6a093201bd7163b48e5b5ab24e705d2cd69e5)) - Angelos Drossos
- **(types)** auto-touch tasks in drop methode - ([fba165c](https://github.com/DevelAngel/keep-it-done/commit/fba165cd2e42b83864071e80803955492b91ccf8)) - Angelos Drossos
- **(types)** add authors field to Task - ([019b9ad](https://github.com/DevelAngel/keep-it-done/commit/019b9ad0b7bbbfd18d15a1971b8446d7dfab790b)) - Angelos Drossos
- **(types)** debounce author timestamps - ([02f8cca](https://github.com/DevelAngel/keep-it-done/commit/02f8cca3ff3723e8b6031af8c0aa024e78588afc)) - Angelos Drossos
- thread actor through TaskCache - ([790684d](https://github.com/DevelAngel/keep-it-done/commit/790684de6e2b364f2e4370204d20558c9ac32003)) - Angelos Drossos
- extract Remote-User header for actor - ([44d0e31](https://github.com/DevelAngel/keep-it-done/commit/44d0e31665c711617fa3051b90139433527990f9)) - Angelos Drossos

### Miscellaneous Chores

- **(data)** [**breaking**] migrate tasks - ([91e4707](https://github.com/DevelAngel/keep-it-done/commit/91e47070e02b3e7b3da8dad43b6075c900964edb)) - Angelos Drossos
- **(dates)** [**breaking**] migrate tasks with different authors and timestamps - ([66608ab](https://github.com/DevelAngel/keep-it-done/commit/66608ab317e90a01e1328409323694c4ea6eb732)) - Angelos Drossos
- **(goosehints)** add UX note - ([bcbd1bf](https://github.com/DevelAngel/keep-it-done/commit/bcbd1bfc0504d72b318ce18f8d3606bf95a980e5)) - Angelos Drossos

### Refactoring

- **(app)** store NaiveDate in DayGrouped - ([1cfad37](https://github.com/DevelAngel/keep-it-done/commit/1cfad375123b64f35e31ca0373622d951967545c)) - Angelos Drossos
- merge touched into author timestamps - ([69739e1](https://github.com/DevelAngel/keep-it-done/commit/69739e1e9b1b2fd8dc6720ee1ffcc0d7b1a0aa97)) - Angelos Drossos
- use IndexMap for Task authors field - ([b66a384](https://github.com/DevelAngel/keep-it-done/commit/b66a384be6c638704410d485b9bd97a51ee88576)) - Angelos Drossos

---
## [0.9.2](https://github.com/DevelAngel/keep-it-done/compare/v0.9.0..v0.9.2) - 2026-04-18

### Bug Fixes

- **(app)** keep task list visible during refetch - ([f22903e](https://github.com/DevelAngel/keep-it-done/commit/f22903e8238ba53ba673e438856fb2ccfc879c53)) - Angelos Drossos
- **(app)** skip EditableField save on blur when value unchanged - ([14ea7b9](https://github.com/DevelAngel/keep-it-done/commit/14ea7b98cae679c953f6a51f975a7f05eda82510)) - Angelos Drossos
- **(app)** stable category chips via shared context - ([6ddc6c7](https://github.com/DevelAngel/keep-it-done/commit/6ddc6c736ee5033163eb0ac15001a78424157f84)) - Angelos Drossos
- **(app)** stable context chips via shared context - ([39e7983](https://github.com/DevelAngel/keep-it-done/commit/39e7983e3a9b658f2029b4bd7026d97c66b8c3b2)) - Angelos Drossos

### Documentation

- **(analysis)** add self-worth analysis (de) - ([6058084](https://github.com/DevelAngel/keep-it-done/commit/6058084c42ab0b269e78adf08f465ca16bab9075)) - Angelos Drossos
- **(analysis)** improve self-worth analysis (de) - ([fc75c77](https://github.com/DevelAngel/keep-it-done/commit/fc75c77600c0422e027d0a38317109a72494c7fd)) - Angelos Drossos
- **(analysis)** self-worth: more focus on AI - ([fddb9ca](https://github.com/DevelAngel/keep-it-done/commit/fddb9ca7e0e42d272fc4401afb5d30564d60ad9c)) - Angelos Drossos
- **(goose)** add project-specific goosehints - ([706e79e](https://github.com/DevelAngel/keep-it-done/commit/706e79ec1ca6f2c936dc90ea644480bc5061e9f9)) - Angelos Drossos

### Features

- **(app)** auto-expand newly created task - ([2330ad0](https://github.com/DevelAngel/keep-it-done/commit/2330ad04ff5d4ba95437d0d7536c7ab905046ffd)) - Angelos Drossos
- **(app)** scroll to newly created task - ([e0283bf](https://github.com/DevelAngel/keep-it-done/commit/e0283bf39aad62830229c755fbe35efc5816ec24)) - Angelos Drossos
- **(app)** scroll to task after category change - ([7e2e2a0](https://github.com/DevelAngel/keep-it-done/commit/7e2e2a0efca3d580154fe6437403a0d1497f42e8)) - Angelos Drossos
- **(app)** show version in footer - ([2e5c48b](https://github.com/DevelAngel/keep-it-done/commit/2e5c48bf3fbb5cbe9afae0d49f71346b3767bf4d)) - Angelos Drossos

### Refactoring

- **(app)** rename filter_suggestion_list - ([60d5a52](https://github.com/DevelAngel/keep-it-done/commit/60d5a52b3504ccca97fc51939984cbaff30de8fd)) - Angelos Drossos
- return Uuid from TaskCache::add() and add_task() - ([d183a0f](https://github.com/DevelAngel/keep-it-done/commit/d183a0f89236b9d0a9ab8877e56de3d39073e955)) - Angelos Drossos

---
## [0.9.0](https://github.com/DevelAngel/keep-it-done/compare/v0.8.0..v0.9.0) - 2026-04-10

### Documentation

- **(ai)** provide SKILL.md - ([3d9604b](https://github.com/DevelAngel/keep-it-done/commit/3d9604bf30c3663b3983ba1de889dae9c62775d9)) - Angelos Drossos

### Features

- **(app)** show contexts as timeline item in task detail view - ([311c3c7](https://github.com/DevelAngel/keep-it-done/commit/311c3c734003215315701fbfa7ce2577d87dc0f1)) - Angelos Drossos
- **(app)** edit contexts in task detail view - ([211dde3](https://github.com/DevelAngel/keep-it-done/commit/211dde3e3426e406d94bd273e0309489dc1a7ca0)) - Angelos Drossos
- **(app)** edit contexts with optimistic UI and per-chip error states - ([25fe8f8](https://github.com/DevelAngel/keep-it-done/commit/25fe8f86047868289a87326b5e22e530c19856d2)) - Angelos Drossos
- **(app)** [**breaking**] add context filter with funnel/pencil toolbar icons - ([1813308](https://github.com/DevelAngel/keep-it-done/commit/1813308caf95ca6a2b943d37cec27f139087f53e)) - Angelos Drossos
- **(cli)** add --context flag to add command - ([79f50bc](https://github.com/DevelAngel/keep-it-done/commit/79f50bc8e66d1c0a20d06671c7f72d0cdfaf79bc)) - Angelos Drossos
- **(cli)** add recategorize command - ([f6e48b0](https://github.com/DevelAngel/keep-it-done/commit/f6e48b085a7ee2d27a18ba554e18fcceb5de5c2c)) - Angelos Drossos
- **(cli)** add replace-contexts command - ([9891c77](https://github.com/DevelAngel/keep-it-done/commit/9891c7750cd330e8f0e3ca924a4be357c4d9d3cb)) - Angelos Drossos
- **(cli)** add add-contexts command - ([6f39ae5](https://github.com/DevelAngel/keep-it-done/commit/6f39ae57743e3a42caf8db6bbfcc3fd7d6955780)) - Angelos Drossos
- **(cli)** add contexts command - ([2fa9823](https://github.com/DevelAngel/keep-it-done/commit/2fa9823151b1bd39a9e59f77335dc3b223e25e8f)) - Angelos Drossos
- **(types)** add contexts field - ([70116d5](https://github.com/DevelAngel/keep-it-done/commit/70116d56f77bd981723de12783430c0202947d30)) - Angelos Drossos

### Miscellaneous Chores

- **(data)** add contexts to tasks - ([db69504](https://github.com/DevelAngel/keep-it-done/commit/db69504b3dd70d8d80d9a78f2d0ca3b45781f229)) - Angelos Drossos

---
## [0.8.0](https://github.com/DevelAngel/keep-it-done/compare/v0.7.1..v0.8.0) - 2026-04-07

### Bug Fixes

- **(app)** show inline error when category update fails - ([88c9714](https://github.com/DevelAngel/keep-it-done/commit/88c971443044b0c569ce8e8ad812f5e2648c06b3)) - Angelos Drossos
- **(app)** show inline error when summary update fails - ([da9a4ed](https://github.com/DevelAngel/keep-it-done/commit/da9a4ed5a300bcbd3782a857aca2fd42ae673300)) - Angelos Drossos
- **(types)** [**breaking**] rewrite legacy context field to category on load - ([c127486](https://github.com/DevelAngel/keep-it-done/commit/c127486fce102a96139b05e464e028b46aa9e272)) - Angelos Drossos

### Documentation

- **(uxdr)** add category-vs-context - ([2d0fd0d](https://github.com/DevelAngel/keep-it-done/commit/2d0fd0d20eddcf3c3e8eaaefb67f531a7bc95d8d)) - Angelos Drossos

### Features

- **(app)** [**breaking**] group task list by category with collapsible groups - ([f55ac64](https://github.com/DevelAngel/keep-it-done/commit/f55ac64a987d2defdc32d3ae379136c44df34724)) - Angelos Drossos
- **(app)** category chips in edit mode - ([eed2c1d](https://github.com/DevelAngel/keep-it-done/commit/eed2c1d4ed480276bfa56922e07d54d8c5349148)) - Angelos Drossos
- **(app)** [**breaking**] introduce Category as mandatory task field with grouped list view - ([722a06c](https://github.com/DevelAngel/keep-it-done/commit/722a06c6213b84349203b66f4a7425c0545f3b2a)) - Angelos Drossos
- **(cli)** add --category as required argument to add command - ([6f904f4](https://github.com/DevelAngel/keep-it-done/commit/6f904f48512440ab9d57ee8dbb59b7e62abbc505)) - Angelos Drossos
- **(cli)** add categories command - ([8825b51](https://github.com/DevelAngel/keep-it-done/commit/8825b51907746f152057295081808cbf87f43e8a)) - Angelos Drossos

### Miscellaneous Chores

- **(data)** [**breaking**] auto-migrationed tasks - ([554d0d2](https://github.com/DevelAngel/keep-it-done/commit/554d0d20f03e90ea76373b038c55602b7f56f4f4)) - Angelos Drossos

### Refactoring

- **(app)** move category field out of timeline into edit section - ([aba5e9c](https://github.com/DevelAngel/keep-it-done/commit/aba5e9ca3b81cf15ed4e754a8bcbd41a9ac42810)) - Angelos Drossos
- **(types)** [**breaking**] rename context field to category - ([0b31250](https://github.com/DevelAngel/keep-it-done/commit/0b312506eb8ae794421ec93e1fa15ebc9b668eef)) - Angelos Drossos
- **(types)** introduce Category newtype - ([071c47a](https://github.com/DevelAngel/keep-it-done/commit/071c47a70e1bd9fb8a9a0cf9fa9cbd7e45d391f1)) - Angelos Drossos
- **(types)** introduce Summary newtype - ([714dd40](https://github.com/DevelAngel/keep-it-done/commit/714dd401165d30b6663c88b063f6bd5edf4e43c9)) - Angelos Drossos

---
## [0.7.1](https://github.com/DevelAngel/keep-it-done/compare/v0.7.0..v0.7.1) - 2026-04-04

### Features

- **(app)** show UUID in task details - ([ba22892](https://github.com/DevelAngel/keep-it-done/commit/ba228920096f9183d0ee1938de804cef00686c6d)) - Angelos Drossos
- **(app)** show last status change only if it differs - ([4ac56f1](https://github.com/DevelAngel/keep-it-done/commit/4ac56f1590ab4c290223df9fc93afe160969ff53)) - Angelos Drossos
- **(app)** add task via inline form in edit mode - ([a0e8672](https://github.com/DevelAngel/keep-it-done/commit/a0e8672914b6fbaaee83c36eb7186ab4031a82f1)) - Angelos Drossos
- **(app)** exit edit mode via Escape - ([2822635](https://github.com/DevelAngel/keep-it-done/commit/2822635c8ea99d31d7c7420429b62557d73e392d)) - Angelos Drossos
- **(app)** add view subtitle with filter and sort info - ([b58b609](https://github.com/DevelAngel/keep-it-done/commit/b58b609ab2c3cd2a8d2dbb90841aa415e480c6eb)) - Angelos Drossos

### Refactoring

- **(app)** per-view server functions with server-side sorting - ([bf1c951](https://github.com/DevelAngel/keep-it-done/commit/bf1c9517265537973b8e6aa12f5ff846f890f5ac)) - Angelos Drossos

---
## [0.7.0](https://github.com/DevelAngel/keep-it-done/compare/v0.6.0..v0.7.0) - 2026-04-03

### Bug Fixes

- **(dates)** 8 auto-migrationed tasks - ([349e6af](https://github.com/DevelAngel/keep-it-done/commit/349e6affc6f716ccb0035fcfae16f6eb5da5e351)) - Angelos Drossos
- **(time-estimate)** auto-migration - ([ec77361](https://github.com/DevelAngel/keep-it-done/commit/ec773611564add219f778b017493cf528e825b4b)) - Angelos Drossos
- **(time-estimate)** 2 auto-migrationed tasks - ([38cf095](https://github.com/DevelAngel/keep-it-done/commit/38cf0959907ddfa012e7edf73590c53cdd39d69c)) - Angelos Drossos

### Documentation

- **(concepts)** add web-edit - ([7f06497](https://github.com/DevelAngel/keep-it-done/commit/7f064972785b21cc2bea468d0ad608ddea584a48)) - Angelos Drossos
- **(uxdr)** add time-estimate-input - ([594c47b](https://github.com/DevelAngel/keep-it-done/commit/594c47bc549828fa5888df1366d169b62ed15df2)) - Angelos Drossos
- **(web-edit)** no floating button - ([40eafce](https://github.com/DevelAngel/keep-it-done/commit/40eafceb63eabd5b9d8c0c7f12321bc59d10f04d)) - Angelos Drossos

### Features

- **(app)** show last status change timestamp in task details - ([98295a4](https://github.com/DevelAngel/keep-it-done/commit/98295a4ca09efbc1f4b5dc49d6fbf5081d9fedb2)) - Angelos Drossos
- **(time-estimate)** [**breaking**] introduce fixed values - ([a14296b](https://github.com/DevelAngel/keep-it-done/commit/a14296b4748a12fc70b0d55ec91d66f6d8a3e944)) - Angelos Drossos
- **(web-edit)** add edit-mode button - ([8cd702c](https://github.com/DevelAngel/keep-it-done/commit/8cd702cec3729455655d289b4d2eb73e7e0a29f0)) - Angelos Drossos
- **(web-edit)** editable context field in edit mode - ([b5a90ec](https://github.com/DevelAngel/keep-it-done/commit/b5a90ecc2da04bc47c7d5755c666e355611262c6)) - Angelos Drossos
- **(web-edit)** editable notes field - ([f5e64bc](https://github.com/DevelAngel/keep-it-done/commit/f5e64bc1a5322d822d2a37ec2617365030143d94)) - Angelos Drossos
- **(web-edit)** editable priority field with 3-button toggle - ([e64936d](https://github.com/DevelAngel/keep-it-done/commit/e64936d77efbef119d6108a61760f390553c54bd)) - Angelos Drossos
- **(web-edit)** editable time estimate field - ([506ab1d](https://github.com/DevelAngel/keep-it-done/commit/506ab1da9d0b6fb0918dec9e508e52889febd771)) - Angelos Drossos
- **(web-edit)** edit mode active -> fixed amber border - ([b89d47c](https://github.com/DevelAngel/keep-it-done/commit/b89d47c0ca3ffcc03be0180c09a37b24fd85f1ab)) - Angelos Drossos
- **(web-edit)** add due date and start date editing - ([b75b693](https://github.com/DevelAngel/keep-it-done/commit/b75b6937594ddf24e65073710c7e6cdca0c55c94)) - Angelos Drossos
- **(web-edit)** make summary editable in edit mode - ([515df1d](https://github.com/DevelAngel/keep-it-done/commit/515df1d4a70857f79051f7a4d24b08b32ddfda46)) - Angelos Drossos
- best-effort migration for legacy values - ([2b46a80](https://github.com/DevelAngel/keep-it-done/commit/2b46a80cae845d481965c8b5c2aa208b5a26f4ed)) - Angelos Drossos

### Miscellaneous Chores

- **(dates)** [**breaking**] auto-migrationed tasks - ([0a4b128](https://github.com/DevelAngel/keep-it-done/commit/0a4b1285cccf3567dcb8d298458576a73aeb6bca)) - Angelos Drossos
- **(time-estimate)** [**breaking**] auto-migrationed tasks - ([ca919a4](https://github.com/DevelAngel/keep-it-done/commit/ca919a47e50dcb27f7d53ba96b5587766f87fe89)) - Angelos Drossos

### Refactoring

- **(dates)** [**breaking**] replace DateEstimation with Date{date,soft} - ([7ec7d8a](https://github.com/DevelAngel/keep-it-done/commit/7ec7d8a930089a31282a50eba5579536b2e55906)) - Angelos Drossos
- **(web-edit)** only edit-mode icon on right side - ([a417e7b](https://github.com/DevelAngel/keep-it-done/commit/a417e7bbbcd4f9ac80b202cad6bb10b476aafd3f)) - Angelos Drossos
- **(web-edit)** add toggle below header - ([c9ae409](https://github.com/DevelAngel/keep-it-done/commit/c9ae4096282a47af618f9061b3829bf440b866f7)) - Angelos Drossos
- **(web-edit)** unify editable fields into component - ([c8b2a74](https://github.com/DevelAngel/keep-it-done/commit/c8b2a7424379462bbea036f1cbaa887054121b85)) - Angelos Drossos

---
## [0.6.0](https://github.com/DevelAngel/keep-it-done/compare/v0.5.0..v0.6.0) - 2026-04-01

### Bug Fixes

- **(app)** expand dot touch target to ~44px via padding - ([3e2454c](https://github.com/DevelAngel/keep-it-done/commit/3e2454ce7b3fdc034217bb480b3d42f24b457281)) - Angelos Drossos

### Documentation

- **(adr)** remove redundant code examples - ([ff400a0](https://github.com/DevelAngel/keep-it-done/commit/ff400a0c96416f356e321fa6cdd32ae04d800278)) - Angelos Drossos
- **(adr)** replace Kanban references with task list - ([d49cce8](https://github.com/DevelAngel/keep-it-done/commit/d49cce8b4f5ea94afca19c022c214ad4093cbf3f)) - Angelos Drossos
- **(adr)** convert to MADR 4.0.0 and add README - ([f5d4bdc](https://github.com/DevelAngel/keep-it-done/commit/f5d4bdc52687b28b25f12a34b8fb82b93775f212)) - Angelos Drossos
- **(concepts)** remove redundant code examples - ([86f3f15](https://github.com/DevelAngel/keep-it-done/commit/86f3f152906f604abdb9a3e182d094e52725602b)) - Angelos Drossos
- **(readme)** add screenshots - ([9523a18](https://github.com/DevelAngel/keep-it-done/commit/9523a18accca2b3e4d9bdab2b3559e52bafc55ba)) - Angelos Drossos
- **(readme)** add link to mental load analysis - ([1d5d1ec](https://github.com/DevelAngel/keep-it-done/commit/1d5d1eca1c97a0057d47be9a38bd24304f8517d4)) - Angelos Drossos
- **(readme)** add privacy note - ([47aed62](https://github.com/DevelAngel/keep-it-done/commit/47aed6238ee0b54e19ac63840a5eaedb273e7f44)) - Angelos Drossos
- **(readme)** update screenshots for all four views - ([66616ec](https://github.com/DevelAngel/keep-it-done/commit/66616ec5e007227503478cfb8c21e9c185c8987b)) - Angelos Drossos
- **(user-guide)** align with current implementation - ([5ae534c](https://github.com/DevelAngel/keep-it-done/commit/5ae534c357ba49f6524c9e73b540b4d8f7f5f1ee)) - Angelos Drossos
- **(uxdr)** convert to MADR 4.0.0 and add README - ([848a940](https://github.com/DevelAngel/keep-it-done/commit/848a9400f5ad5a8d012064efe75fec3ac5ca24ed)) - Angelos Drossos
- sync architecture docs with current implementation - ([67f3fda](https://github.com/DevelAngel/keep-it-done/commit/67f3fda27bf4e683e5cbaded5626b1d2a823a79e)) - Angelos Drossos
- rewrite README with accurate current state - ([f8d9bfa](https://github.com/DevelAngel/keep-it-done/commit/f8d9bfac35ea60fcd6bc042ba2908976080ef214)) - Angelos Drossos

### Features

- **(app)** introduce different views - ([766a3ec](https://github.com/DevelAngel/keep-it-done/commit/766a3ec860c74aeaa09cee56f2d446993bfe5467)) - Angelos Drossos
- **(app)** do not strike-through tasks in WhatIFinished - ([ba6413c](https://github.com/DevelAngel/keep-it-done/commit/ba6413c0a729931c9c40a48f18821f546655243c)) - Angelos Drossos
- **(app)** delay task removal from view - ([eba41ae](https://github.com/DevelAngel/keep-it-done/commit/eba41ae501558ceba0c5a2764a5673beb09e30f5)) - Angelos Drossos
- **(app)** view-dependent completion button colors - ([b183568](https://github.com/DevelAngel/keep-it-done/commit/b18356808a93131519b7a29366271cd20427febc)) - Angelos Drossos
- **(app)** add Quick Wins view - ([e24179f](https://github.com/DevelAngel/keep-it-done/commit/e24179f534e0953e4bd436db7268086b04ca3d02)) - Angelos Drossos
- **(app)** add Recent Changes view - ([b9e229e](https://github.com/DevelAngel/keep-it-done/commit/b9e229e32676e59935143b089c830760253e87ec)) - Angelos Drossos
- **(app)** add view-specific task sorting - ([16ec906](https://github.com/DevelAngel/keep-it-done/commit/16ec906cca4299ce361dad05821e3b58d17b4149)) - Angelos Drossos

### Miscellaneous Chores

- **(claude)** init CLAUDE.md - ([fd8222f](https://github.com/DevelAngel/keep-it-done/commit/fd8222fb13ff2d7763a44aeaae2cf3c2fd53a8b0)) - Angelos Drossos
- **(claude)** optimize CLAUDE.md - ([0ec8af8](https://github.com/DevelAngel/keep-it-done/commit/0ec8af83dd3da7cc6c101b7c79a90698970cfc0c)) - Angelos Drossos
- **(claude)** optimize CLAUDE.md (filler words) - ([4cfdff9](https://github.com/DevelAngel/keep-it-done/commit/4cfdff9e85b76f67f77e11dca58b6d2412d6cdbe)) - Angelos Drossos
- add deno.jsonc - ([8af5fd4](https://github.com/DevelAngel/keep-it-done/commit/8af5fd4a99c9ed681587e291633f28412ee74f7b)) - Angelos Drossos
- add rust-analyzer config - ([0e25fa8](https://github.com/DevelAngel/keep-it-done/commit/0e25fa83d8d947bba6d26ab733d847dd5199f445)) - Angelos Drossos
- update rustfmt config - ([7f755aa](https://github.com/DevelAngel/keep-it-done/commit/7f755aa49d9c94546a545d8822b114f72028ff69)) - Angelos Drossos
- provide example tasks - ([afa1b2c](https://github.com/DevelAngel/keep-it-done/commit/afa1b2c2a7fd5b23fde995dc7a61ff6f6d4a1bb0)) - Angelos Drossos
- update screenshots - ([700bf73](https://github.com/DevelAngel/keep-it-done/commit/700bf737ac68c04525472fffe8f435bb519badc3)) - Angelos Drossos

### Refactoring

- **(app)** replace View::COUNT with strum - ([50fd979](https://github.com/DevelAngel/keep-it-done/commit/50fd979e9ad55f74fa8ed0942923272960c05137)) - Angelos Drossos
- **(app)** replace manual index/from_index - ([3ada95a](https://github.com/DevelAngel/keep-it-done/commit/3ada95ab50aaceb95c0c87668148807ab236990d)) - Angelos Drossos
- server-side task filtering - ([4838f00](https://github.com/DevelAngel/keep-it-done/commit/4838f0012b5c39e8675b7468be017962c6f4a728)) - Angelos Drossos

### Style

- **(app)** adapt view title colors - ([f808b08](https://github.com/DevelAngel/keep-it-done/commit/f808b08acd8eebb6fc11776025569ebe9b8a79a1)) - Angelos Drossos

### Tests

- **(tasks)** complete some tasks - ([35c1a77](https://github.com/DevelAngel/keep-it-done/commit/35c1a774cdcb7653d39b4c9ec28227e91f40b519)) - Angelos Drossos

---
## [0.5.0](https://github.com/DevelAngel/keep-it-done/compare/v0.4.0..v0.5.0) - 2026-03-14

### Features

- **(app)** adapt colors to favicon colors - ([3511dd9](https://github.com/DevelAngel/keep-it-done/commit/3511dd910d29f0e4c766398e73310355fec49b1f)) - Angelos Drossos
- **(app)** adapt colors to favicon colors (2) - ([30ceb02](https://github.com/DevelAngel/keep-it-done/commit/30ceb02dc6c28aa5dae131f150d22a79a75eba0c)) - Angelos Drossos
- **(app)** adapt colors to favicon colors (3) - ([04dbe76](https://github.com/DevelAngel/keep-it-done/commit/04dbe76560e4759e267ced41be1044ccaa312e61)) - Angelos Drossos
- **(status)** track status change - ([831dd61](https://github.com/DevelAngel/keep-it-done/commit/831dd61c9133361388ca30549d453986b8d904ca)) - Angelos Drossos
- **(status)** detect dirty loads - ([608199b](https://github.com/DevelAngel/keep-it-done/commit/608199b419f7f2575d938b73974a6babdc559676)) - Angelos Drossos
- new favicon - ([7dbfdf7](https://github.com/DevelAngel/keep-it-done/commit/7dbfdf7ed69a7e4f2fc33c0685fe5544031f639e)) - Angelos Drossos

### Refactoring

- **(types)** print load error details - ([e60f070](https://github.com/DevelAngel/keep-it-done/commit/e60f0704a0fb8095a06febcd996f38d8862a5b86)) - Angelos Drossos

---
## [0.4.0] - 2026-03-08

### Bug Fixes

- **(cross)** POST 404 Not Found - ([518ac2b](https://github.com/DevelAngel/keep-it-done/commit/518ac2befb3462e122a1feb2d7908d9b99381d1a)) - Angelos Drossos
- clippy findings - ([5e9c49d](https://github.com/DevelAngel/keep-it-done/commit/5e9c49d28be03f98d2b3f66473f62a4a900a4814)) - Angelos Drossos

### Build

- **(types)** improve cond. compilation (miette) - ([caed8e4](https://github.com/DevelAngel/keep-it-done/commit/caed8e41e268f7b977e552f97a2f82a6e6a93f80)) - Angelos Drossos
- activate leptos tracing feature - ([8ee5305](https://github.com/DevelAngel/keep-it-done/commit/8ee530522392dd33f28355a05c8eb52c8ddd24d3)) - Angelos Drossos
- support aarch64 build - ([d931d2c](https://github.com/DevelAngel/keep-it-done/commit/d931d2c20ccb992338093272ea6c10fb9bff6e1c)) - Angelos Drossos
- update dependency versions - ([b359bc9](https://github.com/DevelAngel/keep-it-done/commit/b359bc94c049ab6da3f5918446872ecbd575099e)) - Angelos Drossos
- add --locked - ([f21fbbf](https://github.com/DevelAngel/keep-it-done/commit/f21fbbf6171f80e7076b4ba3e52e95c768374076)) - Angelos Drossos

### Documentation

- **(adr)** add cli-argument-parsing - ([805a045](https://github.com/DevelAngel/keep-it-done/commit/805a045a454a610f356397a21cc335ffdf4bd498)) - Angelos Drossos
- **(adr)** add task storage ADR - ([ffc9a42](https://github.com/DevelAngel/keep-it-done/commit/ffc9a42c45502edee3458701a976cc62598da5f8)) - Angelos Drossos
- **(adr)** add leptos browser-based ui - ([fdac1bc](https://github.com/DevelAngel/keep-it-done/commit/fdac1bc06b1b85e13252e5edecd83662eef3cfd1)) - Angelos Drossos
- **(adr)** add tarpc for rust-native rpc-communication - ([89477dd](https://github.com/DevelAngel/keep-it-done/commit/89477ddbc784f9dcf4ec5b3665a804e7a89d0989)) - Angelos Drossos
- **(adr)** add checkbox toggle - ([68ef2f0](https://github.com/DevelAngel/keep-it-done/commit/68ef2f09612931352c08bdfa06daf36730aa065c)) - Angelos Drossos
- **(adr)** add tailwindcss - ([c91aa22](https://github.com/DevelAngel/keep-it-done/commit/c91aa22f561a7e71437e067f393c77dae3718c5b)) - Angelos Drossos
- **(adr)** add cli-api-design - ([e367aec](https://github.com/DevelAngel/keep-it-done/commit/e367aece96a386872663bd08fb85c11094ff84e2)) - Angelos Drossos
- **(adr)** add tinyauth deployment - ([5c50876](https://github.com/DevelAngel/keep-it-done/commit/5c50876ec4213a1d05e7a2434cd34aaeb1574d92)) - Angelos Drossos
- **(analysis)** change mental load version to 1.3 - ([7526002](https://github.com/DevelAngel/keep-it-done/commit/75260025c0a85fca421ca3efc5fbd623a79e6787)) - Angelos Drossos
- **(analysis)** add writing style guide - ([02d6728](https://github.com/DevelAngel/keep-it-done/commit/02d67289ad4f19eea3d480e9c4ce61507883bec0)) - Angelos Drossos
- **(analysis)** add writing style guide for KI - ([ca98970](https://github.com/DevelAngel/keep-it-done/commit/ca989702585a8c417540409226fb565767e1e2c5)) - Angelos Drossos
- **(analysis)** add writing style guide for KI (en) - ([8865704](https://github.com/DevelAngel/keep-it-done/commit/886570442b8c00e7eec670a304c9ec0b854b047b)) - Angelos Drossos
- **(concept)** add task card concept - ([861da3f](https://github.com/DevelAngel/keep-it-done/commit/861da3fa8345bdfaf3dd860ea4528fdfa34c61ce)) - Angelos Drossos
- **(concept)** add task storage concept - ([653fb9a](https://github.com/DevelAngel/keep-it-done/commit/653fb9a2bbc82998b03abf2e5eb4f8f500530f71)) - Angelos Drossos
- **(design)** add task-detail design variants - ([7de8f20](https://github.com/DevelAngel/keep-it-done/commit/7de8f201109e54fb0c66defabbfea25762e6b114)) - Angelos Drossos
- **(design)** rename task detail variants - ([945d445](https://github.com/DevelAngel/keep-it-done/commit/945d4450da3704dce915b404436a764cf232d01a)) - Angelos Drossos
- **(design)** add task view switching - ([6278862](https://github.com/DevelAngel/keep-it-done/commit/627886210eb53c3283121cb5d7faa58bed2a3774)) - Angelos Drossos
- **(uxdr)** add timeline detail expansion - ([460fe30](https://github.com/DevelAngel/keep-it-done/commit/460fe30fad81cddb5c4172918fd17f6d9337fadf)) - Angelos Drossos
- **(uxdr)** add view switching - ([5d935b1](https://github.com/DevelAngel/keep-it-done/commit/5d935b1210bb287b3a4e7447879d5823b3d6b371)) - Angelos Drossos
- add architecture overview - ([fcdf402](https://github.com/DevelAngel/keep-it-done/commit/fcdf402f3cadab1bb2fce40fcb757a12b47c72cd)) - Angelos Drossos
- add user guide - ([387fad3](https://github.com/DevelAngel/keep-it-done/commit/387fad370bec32305bc1c895f9e303923b518335)) - Angelos Drossos
- write README - ([34d5b93](https://github.com/DevelAngel/keep-it-done/commit/34d5b933fd8821775689f1a0a564bb23aecdcb83)) - Angelos Drossos
- add mental load analysis v1 - ([28f27d1](https://github.com/DevelAngel/keep-it-done/commit/28f27d18d780618f5bb3a060608fbf1a9c78cb52)) - Angelos Drossos
- add mental load analysis v2 - ([e59a722](https://github.com/DevelAngel/keep-it-done/commit/e59a722e37925066849656f9176960955a6d5680)) - Angelos Drossos
- add mental load analysis v3 - ([408a7a9](https://github.com/DevelAngel/keep-it-done/commit/408a7a92e6adf56b9aec53daffa24aba8730951d)) - Angelos Drossos
- add analysis about mental load - ([22dbe0e](https://github.com/DevelAngel/keep-it-done/commit/22dbe0e6512b6195084ef62675164e63bd22c510)) - Angelos Drossos
- add deployment instructions - ([54343d7](https://github.com/DevelAngel/keep-it-done/commit/54343d7bdda006a12cd704d8f9f897531e6b18a6)) - Angelos Drossos

### Features

- **(app)** style task list - ([7c628ef](https://github.com/DevelAngel/keep-it-done/commit/7c628efe2cd359af3a639e610807043076fc022b)) - Angelos Drossos
- **(app)** timeline-style task detail expansion - ([1d9e372](https://github.com/DevelAngel/keep-it-done/commit/1d9e372c7b0c81a718114ec82a98e9f064adaf08)) - Angelos Drossos
- **(cache)** stabilize cache order - ([fb55884](https://github.com/DevelAngel/keep-it-done/commit/fb55884905eceb037263dd11c917003db100f6c6)) - Angelos Drossos
- **(cli)** add cli crate - ([a28355c](https://github.com/DevelAngel/keep-it-done/commit/a28355c281c02b4dda637b116a6ae98fead5e0cf)) - Angelos Drossos
- **(cli)** provide task json schema - ([8890137](https://github.com/DevelAngel/keep-it-done/commit/889013702f25a13e64e0eac3c13dd93707f8f8eb)) - Angelos Drossos
- **(cli)** add command: set details via JSON - ([753d30f](https://github.com/DevelAngel/keep-it-done/commit/753d30fac3f22ac5bf25c470962f5bb91ff28673)) - Angelos Drossos
- **(cli)** print task list as json - ([50dd25e](https://github.com/DevelAngel/keep-it-done/commit/50dd25e84241da61c08cee20c84722e3c1ce36f6)) - Angelos Drossos
- **(cli)** add task complete - ([0f941ec](https://github.com/DevelAngel/keep-it-done/commit/0f941ece17c0652d33fe362e8f82fd1508c07f37)) - Angelos Drossos
- **(cli)** support env settings - ([d3b5f5a](https://github.com/DevelAngel/keep-it-done/commit/d3b5f5a9d5303dc8be8003a20ee09f484b76188f)) - Angelos Drossos
- **(frontend)** enable web-console logging - ([893cb01](https://github.com/DevelAngel/keep-it-done/commit/893cb01e6d678b31525a9019c0bdb87b6c03d935)) - Angelos Drossos
- **(rpc)** access same task cache - ([c1ea398](https://github.com/DevelAngel/keep-it-done/commit/c1ea398f9bfff7dc871fddba7b058bc24cda8e95)) - Angelos Drossos
- **(rpc)** add rename - ([c09bb72](https://github.com/DevelAngel/keep-it-done/commit/c09bb72ac54a7bd134c704447be2b850a1f4e9bd)) - Angelos Drossos
- **(rpc)** add update func - ([11720a0](https://github.com/DevelAngel/keep-it-done/commit/11720a0f89e67307bb506fdbe1c8ac38dadfc0b6)) - Angelos Drossos
- **(rpc)** differ between replace and update - ([2ec8386](https://github.com/DevelAngel/keep-it-done/commit/2ec8386c9dd6d5270483abb3e98233ea17fa05fe)) - Angelos Drossos
- **(server)** track task changes in cache - ([24ab79b](https://github.com/DevelAngel/keep-it-done/commit/24ab79b3a871ece4899d7d49f3b27caa311ee126)) - Angelos Drossos
- **(server)** provide flushing of tasks - ([35ae1dd](https://github.com/DevelAngel/keep-it-done/commit/35ae1ddf5b8276d20f01b4a1e2a2a8789431518e)) - Angelos Drossos
- **(server)** flush tasks every minute - ([14b78ac](https://github.com/DevelAngel/keep-it-done/commit/14b78acfaae5247bb06be16a35ab5461d20bb169)) - Angelos Drossos
- **(server)** graceful shutdown - ([3139426](https://github.com/DevelAngel/keep-it-done/commit/3139426803c815744b7cfe8b6be11d44e78ac78e)) - Angelos Drossos
- **(server)** load tasks - ([f0c6c9e](https://github.com/DevelAngel/keep-it-done/commit/f0c6c9e09218f80cb7a7e4c6d32ca040daf3b222)) - Angelos Drossos
- **(server)** [**breaking**] remove hardcoded tasks - ([54572c1](https://github.com/DevelAngel/keep-it-done/commit/54572c164ff2c66ed6e0588cdc374eb06a940fed)) - Angelos Drossos
- **(types)** use AHash with IndexSet - ([669dc8f](https://github.com/DevelAngel/keep-it-done/commit/669dc8f4cad3651a6f2e628b38a1a5f6d7aa6f0d)) - Angelos Drossos
- **(types)** add completed property - ([82d8aa8](https://github.com/DevelAngel/keep-it-done/commit/82d8aa85d6a4efee43b5722eadb7c3db8383d0ab)) - Angelos Drossos
- add task list rpc communication - ([c0e5079](https://github.com/DevelAngel/keep-it-done/commit/c0e5079455e023dcc5b439cd5e6d25a3ef9ed4a1)) - Angelos Drossos
- add leptos-hydrate - ([7710a9d](https://github.com/DevelAngel/keep-it-done/commit/7710a9dde794f71fcc44501152e2fbab0f35ca9b)) - Angelos Drossos
- add task list http communication - ([3e39d69](https://github.com/DevelAngel/keep-it-done/commit/3e39d6982ed003a8466e20594b659a155cec948a)) - Angelos Drossos
- activate tailwindcss - ([ca4320e](https://github.com/DevelAngel/keep-it-done/commit/ca4320ee93627ee277df834c9984dbba745d5f7b)) - Angelos Drossos
- show task details on click - ([c00f5c0](https://github.com/DevelAngel/keep-it-done/commit/c00f5c098694c2f9f744f2478bf44dc7e2498784)) - Angelos Drossos
- change task complete status - ([5cb9e9d](https://github.com/DevelAngel/keep-it-done/commit/5cb9e9d6619ec2c6e09c048d1962c815962f2cab)) - Angelos Drossos
-  [**breaking**]add more task properties - ([a51791e](https://github.com/DevelAngel/keep-it-done/commit/a51791e0416700c0a8af8beb3fef834e0334979d)) - Angelos Drossos
-  [**breaking**]show task details - ([adaea26](https://github.com/DevelAngel/keep-it-done/commit/adaea2609bb57e070b95df820eee9a1603218187)) - Angelos Drossos
- change RPC default port to 9000 - ([be34b83](https://github.com/DevelAngel/keep-it-done/commit/be34b838f64bdf3029cc3237cc2d0e7a75dd98ca)) - Angelos Drossos

### Miscellaneous Chores

- **(concept)** remove concept from filename - ([d83be8c](https://github.com/DevelAngel/keep-it-done/commit/d83be8c76448a33b9d59ae83f923f47c75f1572a)) - Angelos Drossos
- **(docs)** rename analysis template - ([b18fd3b](https://github.com/DevelAngel/keep-it-done/commit/b18fd3b0f2ffaaff5b34368291ea37e8ec42cfeb)) - Angelos Drossos
- **(docs)** add typst README - ([c2fb88f](https://github.com/DevelAngel/keep-it-done/commit/c2fb88f2b50cf2658dcc1c6498197ae61e2d9e14)) - Angelos Drossos
- **(docs)** reorder writing style guides - ([ae38a68](https://github.com/DevelAngel/keep-it-done/commit/ae38a6891d9339e77e95fb17424773c656ecc002)) - Angelos Drossos
- **(public)** add favicon - ([6534b47](https://github.com/DevelAngel/keep-it-done/commit/6534b477de65a60bf3c427d7659f32027f72d0cf)) - Angelos Drossos
- **(server)** remove sleep from list rpc command - ([b86bcd3](https://github.com/DevelAngel/keep-it-done/commit/b86bcd3b73a0e9cf6c708654f92e4cfcbae6f47f)) - Angelos Drossos
- add rustfmt config - ([395ee68](https://github.com/DevelAngel/keep-it-done/commit/395ee6874e1d8e78771410b394e922b428c0ecac)) - Angelos Drossos
- simplify Cargo.toml - ([f91615a](https://github.com/DevelAngel/keep-it-done/commit/f91615a4d575878e1ac040df64a28b6496f16b1f)) - Angelos Drossos
- move BUILD.md into docs folder - ([2a804a0](https://github.com/DevelAngel/keep-it-done/commit/2a804a0bc9861633969f989384cd01c241704c6d)) - Angelos Drossos

### Refactoring

- **(app)** move server fnc into own module - ([898ebc8](https://github.com/DevelAngel/keep-it-done/commit/898ebc8e333ec43c48f4d927cce8b4f27f59a1cb)) - Angelos Drossos
- **(server)** improve graceful shutdown - ([70f1ec6](https://github.com/DevelAngel/keep-it-done/commit/70f1ec6148dcb7478059f8e9961043ce9f6f9f54)) - Angelos Drossos
- **(server)** improve read/write task files - ([4da08d0](https://github.com/DevelAngel/keep-it-done/commit/4da08d0e855472bb77eb02779349739e4f4d376b)) - Angelos Drossos
- **(types)** use map(id, task) - ([2a643a7](https://github.com/DevelAngel/keep-it-done/commit/2a643a7da59cf5fbbd35ced5264656bf4b12f294)) - Angelos Drossos

<!-- generated by git-cliff -->
