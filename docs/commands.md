# wizard
## show wizard
```
./beans-rs
./beans-rs wizard
```

## don't prompt for user input once done
```
./beans-rs --no-pause
./beans-rs --no-pause wizard
```

## show wizard and use custom location
```
./beans-rs --location <sdk location>
./beans-rs wizard --location <sdk location>
```

## show wizard and use custom location, and don't ask for user input once done
```
./beans-rs --no-pause --location <sourcemods location>
./beans-rs --no-pause wizard --location <sourcemods location>
```

# install
## install or reinstall to default sourcemods location
```
./beans-rs install
```

## install or reinstall to the specified sourcemods location
```
./beans-rs install --location <sourcemods location>
```

## install or reinstall to the specified sourcemods location, and dont ask for user input once done
```
./beans-rs --no-pause install --location <sourcemods location>
```

## install to default location from file specified
```
./beans-rs install --from <.tar.zstd file>
```

## install to the specified sourcemods location from file specified
```
./beans-rs install --from <.tar.zstd file> --location <sourcemods location>
```

## install v18 to the default sourcemods folder
```
./beans-rs install --target-version 18
```

## install to default location from file specified
```
./beans-rs install --from <.tar.zstd file>
```

## install to default location from file specified, and dont ask for user input once done
```
./beans-rs --no-pause install --from <.tar.zstd file>
```

## install to the specified sourcemods location from the file specified and don't ask for user input once done
```
./beans-rs --no-pause install --from <.tar.zstd file> --location <sourcemods location>
```

# update
## update default sourcemods location
```
./beans-rs update
```

## update default sourcemods location and don't ask for user input once done
```
./beans-rs --no-pause update
```

## update specified sourcemods location
```
./beans-rs update --location <sourcemods location>
```

## update specified sourcemods location and dont ask for user input once done
```
./beans-rs --no-pause update --location <sourcemods location>
```

# verify
## verify default sourcemods location
```
./beans-rs verify
```

## verify default sourcemods location and don't ask for user input once done
```
./beans-rs --no-pause verify
```

## verify specified sourcemods location
```
./beans-rs verify --location <sourcemods location>
```

## verify specified sourcemods location and dont ask for user input once done
```
./beans-rs --no-pause verify --location <sourcemods location>
```
