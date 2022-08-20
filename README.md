# bevy_godot
A crate for using [Bevy](https://github.com/bevyengine/bevy) with the [Godot Engine](https://godotengine.org). This crate is in active development and is not ready for production use.

## Features
- Godot SceneTree integration
- Load Godot Resources as a Bevy Asset
- Spawn Godot scenes from Bevy
- Detect Godot object collisions
- Systems can be scheduled for the visual or physics frame
- Tracing behind the `trace` and `trace_chrome` feature flags

## Quickstart
Browse the examples to get a feel for the API. The examples are `cargo run`-able if a `godot` executable is present in your enviroment path.

For a new Godot project, use one of the examples as a starting point. This crate requires the provided `Autoload` class to be added to the project's autoloads.

## Supported Godot Versions
This library depends on [godot-rust](https://github.com/godot-rust/godot-rust) for the Godot API and follows their [compatibility guidelines](https://github.com/godot-rust/godot-rust#engine-compatibility).
