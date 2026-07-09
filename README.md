Command line tool to batch roll [Maprando](https://maprando.com/) seeds.

The tool continuously rolls seeds and compares the Spoiler Logs with a given Lua Script. The winning Spoiler Log is then returned along with the random seed used to generate it. The random seed can then be used to roll the seed properly on the Maprando website.

## How to use

Download the latest release (Windows only currently) from the [Release page](https://github.com/Noktuska/seed-roller/releases) and extract it.
Call the extracted executable from a command line with the following arguments:
- settings-path: Path to a JSON-File containing Randomizer Settings to use.
- map-layout: Vanilla/Standard/Wild, Vanilla by default, choose a map pool to roll from.
- lua-path: Path to a Lua-File used to compare Spoiler Logs. (Optional)
- attempts: Number of total seeds rolled before stopping. (Default: 100)
- attempts-per-seed: Number of attempts to try and roll a successful seed on any given random seed. The Maprando website has this set to 2000, but times out after a certain time. Increasing this limit may result in a successful roll that may not be replicable on the Maprando website due to the timeout. (Default: 100)
- attempts-per-map: Number of attempts to try and roll a seed on a given map before rerolling the map. (Default: 10)
- threads: Number of threads used to parallelize seed rolling. (Default: 1)
- stop_on_success: Stop rolling seeds as soon as the provided Lua Script returns true (a successful seed). (Default: false)
- random_seed: Provides a random seed used to generate the seed instead of generating new seeds on every attempt. (Optional)
- help: Display list of possible arguments.

### Acquiring a Randomizer Settings JSON-File

You can acquire the required Randomizer Settings JSON-File by going to the stable [Maprando website](https://maprando.com/generate):
- Set up the desired settings
- Click on the `Save Settings` button at the bottom and name the preset anything you'd like
- Click on the cog wheel next to the `Settings preset` field
- Click the download button of the preset that was just created

## Lua Scripts

A few examples of Lua Scripts are provided in the `lua_examples` directory. The script has to return a function taking two parameters: `new_spoiler` and `older_spoiler`. `new_spoiler` will be the Spoiler Log of currently rolled seed and will be compared to `old_spoiler`, the best seed rolled so far (`nil` initially). The function should return `true` or `false`, `true` if the seed satisfies any desired conditions and outperforms `old_spoiler` in desired metrics, the `old_spoiler` will then be replaced by the new one in subsequent rolls.

## Building from source

You can clone the repository and build the tool from source using Cargo and Rust. You will need to update the `Cargo.toml` and link to a local clone of the `maprando` and `maprando-game` repository on stable release, which can be found [here](https://github.com/blkerby/MapRandomizer/tree/455d11a28579bf26458fd1bd68f1ac436a70d833).
Additional provide a `data` directory in the project root containing from the Maprando repository:
- room_geometry.json
- visualizer
- TitleScreen
- sm-json-data
- patches
- maps
- A directory called maprando-data containing the `rust/data` directory from the Maprando repository.
Additionally the latest Mosaic patches for Maprando should be built, as well as the Map pool downloaded. Follow the README on the [Maprando repository](https://github.com/blkerby/MapRandomizer/tree/455d11a28579bf26458fd1bd68f1ac436a70d833) for this.
Finally, run the tool using `cargo run --release -- <args>`, where `<args>` are the arguments to be provided to the tool as explained in the section above.