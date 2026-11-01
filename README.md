# Farmer's Delight — Horizon port

A port of [Farmer's Delight](https://github.com/vectorwing/FarmersDelight) 1.3.4 (Minecraft 1.21.1, NeoForge)
by **vectorwing** to [Horizon](https://playhorizon.dev), Minecraft Java Edition 26.3 rewritten in Rust.
The original is MIT-licensed; its copyright and licence notice are kept in [LICENSE](LICENSE).
This port is not made or endorsed by vectorwing.

Порт мода [Farmer's Delight](https://github.com/vectorwing/FarmersDelight) 1.3.4 (автор — **vectorwing**,
лицензия MIT) для [Horizon](https://playhorizon.dev): Minecraft Java 26.3, переписанного на Rust.

## What is in it

Two parts, both needed:

| Project | Kind | What it holds |
| --- | --- | --- |
| `farmersdelight` | content mod (`.hmod`) | Every block, item, texture, model, sound, translation, recipe, loot table, tag and worldgen file of Farmer's Delight 1.3.4 |
| `farmersdelight_native` | native mod (`.hnmod`) | The mechanics, ported from the Java classes: cooking pot (menu, recipes, meals, heat), stove, skillet, cutting board, crops (rice in water, tomatoes, cabbages, onions, wild crops), rich soil, ropes, tatami, canvas signs, food effects and tooltips, bone meal, and the creative tab |

**Works on Horizon Nightly 2026-10-31 (`9d979ca`)** — Работает на Horizon Nightly 2026-10-31 (`9d979ca`).

The native mod is pinned to that Nightly (`9d979caa48a3bb58203319b9ea044db58c38e0d8`): the launcher builds it
into its own game. Versions are named `<Farmer's Delight version>-horizon.<Horizon commit>`.

## Not ported yet

Throwing rotten tomatoes, the Nourishment and Comfort effects' behaviour (they show in tooltips), tomatoes climbing ropes,
the basket picking up items.

## Multiplayer

Single player and Open to LAN work. A server needs the same Horizon build with both parts, and players join it with
the same build and mods. Vanilla Java clients can't join a server that runs content mods: they don't know the new
blocks.

## Building from source

- Content mod: `cargo horizon-mod build` in `content/farmersdelight`.
- Native mod: `native/farmersdelight_native` is a horizon-mixins bundle (`mixins/farmersdelight_native/horizon-mixin.toml`);
  `horizon-launcher native-build native/farmersdelight_native/mixins/farmersdelight_native --trust-native-code` builds it.

## Licence

MIT, as the original: see [LICENSE](LICENSE).
