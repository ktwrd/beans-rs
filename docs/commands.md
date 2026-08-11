# wizard
## show wizard
```
./beans
./beans wizard
```

## don't prompt for user input once done
```
./beans --no-pause
./beans --no-pause wizard
```

## show wizard and use custom location
```
./beans --location <sdk location>
./beans wizard --location <sdk location>
```

## show wizard and use custom location, and don't ask for user input once done
```
./beans --no-pause --location <sourcemods location>
./beans --no-pause wizard --location <sourcemods location>
```

# install
## install or reinstall to default sourcemods location
```
./beans install
```

## install or reinstall to the specified sourcemods location
```
./beans install --location <sourcemods location>
```

## install or reinstall to the specified sourcemods location, and dont ask for user input once done
```
./beans --no-pause install --location <sourcemods location>
```

## install to default location from file specified
```
./beans install --from <.tar.zstd file>
```

## install to the specified sourcemods location from file specified
```
./beans install --from <.tar.zstd file> --location <sourcemods location>
```

## install v18 to the default sourcemods folder
```
./beans install --target-version 18
```

## install to default location from file specified
```
./beans install --from <.tar.zstd file>
```

## install to default location from file specified, and dont ask for user input once done
```
./beans --no-pause install --from <.tar.zstd file>
```

## install to the specified sourcemods location from the file specified and don't ask for user input once done
```
./beans --no-pause install --from <.tar.zstd file> --location <sourcemods location>
```

# update
## update default sourcemods location
```
./beans update
```

## update default sourcemods location and don't ask for user input once done
```
./beans --no-pause update
```

## update specified sourcemods location
```
./beans update --location <sourcemods location>
```

## update specified sourcemods location and dont ask for user input once done
```
./beans --no-pause update --location <sourcemods location>
```

# verify
## verify default sourcemods location
```
./beans verify
```

## verify default sourcemods location and don't ask for user input once done
```
./beans --no-pause verify
```

## verify specified sourcemods location
```
./beans verify --location <sourcemods location>
```

## verify specified sourcemods location and dont ask for user input once done
```
./beans --no-pause verify --location <sourcemods location>
```
