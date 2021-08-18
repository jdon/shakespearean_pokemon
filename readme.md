## Pokedex API

Uses [pokeapi](https://pokeapi.co/) and [funtranslations](https://funtranslations.com)

### Endpoints:

#### **/pokemon/<pokemon_name>**
Example:
```
http://localhost:5000/pokemon/charizard
```
Output:
```
{
    "name": "charizard",
    "description": "Spits fire that is hot enough to melt boulders.Known to cause forest fires unintentionally.",
    "isLegendary": false,
    "habitat": "mountain"
}
```



#### **/pokemon/translated/<pokemon_name>**
Example:
```
http://localhost:5000/pokemon/translated/charizard
```
Output:
```
{
    "name": "charizard",
    "description": "Spits fire yond is hot enow to melt boulders. Known to cause forest fires unintentionally.",
    "isLegendary": false,
    "habitat": "mountain"
}
```


### Build/Testing/Running
Ensure you have working rust install. If you don't you can install it by following these [instructions](https://www.rust-lang.org/tools/install).

This API has been built using rust stable.

#### Build
```
cargo build
```
#### Test
```
cargo test
```
#### Run
Relies on a `port` , `pokemon_api_base_url` and `translation_api_base_url` environment variable:
```
port=5000 pokemon_api_base_url=https://pokeapi.co translation_api_base_url=https://api.funtranslations.com cargo run -- release
```

### Environment variables:
```
port: u16,
api_token: Option<String>
pokemon_api_base_url: String
translation_api_base_url: String
```
If the optional ones aren't specified then a default value will be used.

### Docker
This project can be ran in docker:
1. Create a `.env` file containing the above environment variables.
2. Build docker file: `docker build -t shakespearean_pokemon:0.2 .`
   1. Uses a multistage build with cargo chef caching.
3. Run docker container `docker run -it --init -p 5000:5000 --env-file ./.env shakespearean_pokemon:0.2`

The docker container is also built and hosted on github, so if you don't want to build locally you can pull the image following these [instructions](https://github.com/jdon/shakespearean_pokemon/packages/666939).

### Further Improvements
1. Error Handling and logging
   
   Currently I'm not capturing any additional context when returning an error.
   My errors could be more specific and I could add logging when an error happens.

2. Project layout
   
   Currently this is being built as 1 binary and is tightly coupled. I could turn this into a cargo workspace and split the api clients into library crates, so they can be reused elsewhere.

3. Caching

   There are less than a 1000 pokemon, so we are going to get lots of calls for the same pokemon. The pokemon data and translation are unlikely to change, so we could cache them all to reuse in subsequent requests.