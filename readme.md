## Shakespearean Pokemon
An API server which takes in the name of a pokemon and returns the description in Shakespearean.

Uses [pokeapi](https://pokeapi.co/) and [funtranslations](https://funtranslations.com/api/shakespeare)

### Endpoints
```
/pokemon/<pokemon_name>
```
Example:
```
http://localhost:5000/pokemon/charizard
```
Output:
```
{
	"name":"charizard",
	"description":"Spits fire yond is hot enow to melt boulders. Known to cause forest fires unintentionally."
}
```


### Build/Testing/Running
Ensure you have working rust install. If you don't you can install it by following these [instructions](https://www.rust-lang.org/tools/install).

This API has been built using rust 1.50.0 but is being tested in CI with a minimum rust version of 1.45.0.

#### Build
```
cargo build
```
#### Test
```
cargo test
```
#### Run
Relies on a `port` environment variable:
```
port=5000 && cargo run
```

### Environment variables:
```
port: u16,
api_token: Option<String>
pokemon_api_base_url: Option<String>
shakespeare_api_base_url: Option<String>
```
If the optional one aren't specified then a default value will be used.

### Docker
This project can be ran in docker:
1. Create a `.env` file containing the above environment variables.
2. Build docker file: `docker build -t shakespearean_pokemon:0.1 .`
   1. Uses a multistage build with cargo chef caching.
3. Run docker container `docker run -it --init -p 5000:5000 --env-file ./.env shakespearean_pokemon:0.1`

The docker container is also built and hosted on github, so if you don't want to build locally you can pull the image following these [instructions](https://github.com/jdon/shakespearean_pokemon/packages/666939).

### Further Improvements
1. Error Handling and logging
   
   Currently I'm not capturing any additional context when returning an error.
   My errors could be more specific and I could add logging when an error happens.

2. Project layout
   
   Currently this is being built as 1 binary and is tightly coupled. I could turn this into a cargo workspace and split the api clients into library crates, so they can be reused elsewhere.