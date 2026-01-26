# Contributing

**When creating a PR, you must branch off the `develop` branch.** When merging back into this repo remember to select the `develop` as the branch to merge into. After the **7th of June 2024** any PRs that **do not** use `develop` as the base branch will be closed.

When adding a new feature (that a user will interact with), create a new file in `src/workflows/` with the name of the feature (for example, `launch.rs`). Inside of `launch.rs` you would have a struct with the name of `LaunchWorkflow`. It would look something like this;
```rust
use crate::{RunnerContext, BeansError};

#[derive(Debug, Clone)]
pub struct LaunchWorkflow {
    pub context: RunnerContext
}
impl LaunchWorkflow {
    pub async fn wizard(ctx: &mut RunnerContext) -> Result<(), BeansError>
    {
        todo!("Logic for handling the LaunchWorkflow")
    }
}
```
You would also be adding a subcommand for this ins `main.rs`. In `Launcher::run()` you would add the following ***before*** `.args` is called on `cmd`, and after the last `.subcommand()` that is used. What you would add would look like the following;
```rust
.subcommand(Command::new("launch")
    .about("Launch the currently installed game")
    .arg(Launcher::create_location_arg()))
```

All sub-commands must have the `--location` argument added so the end-user can specify if they have a custom location for their `sourcemods` folder.

Next you'd add a match case so `Launcher::subcommand_processor(&mut self)`, which would look like the following;
```rust
Some(("launch", install_matches)) => {
    self.task_launch(install_matches).await;
}
```

Then, you'd add a new function to `Launcher`, which would actually call `LaunchWorkflow`. It would look something like the following (if there is only the `--location` argument);
```rust
pub async fn task_launch(&mut self, matches: &ArgMatches) {
    self.to_location = Launcher::find_arg_sourcemods_location(&matches); // must be done when the `--launcher` argument is provided on the subcommand!
    if let Err(e) = LaunchWorkflow::wizard(&mut ctx).await {
        panic!("Failed to run LaunchWorkflow {:#?}", e);
    } else {
        logic_done(); // must be called when any flow of logic has completed.
    }
}
```
### Errors
When handling errors, utilize the `BeansError` system in favor of other error instances.
Any necessary external errors should be nested inside of a new `BeansError`.
```rs 
#[error("Failed to serialize provided AppVarData to JSON. ({error:})")]
AppVarDataSerializeFailure
{
    error: serde_json::Error, // <-- Outside Error
    data: AppVarData
},
#[error("Failed to read file attributes on {location} ({error:})")]
ReadFileAttributesError
{
    error: std::io::Error,
    location: String,
    backtrace: Backtrace
},
#[error("Failed to open VPK file at {location} ({error:})")]
VpkOpenFailure
{
    location: String,
    error: anyhow::Error,
    backtrace: Backtrace
}
```
**Do not make any PRs to remove the embedded executables in favor of downloading.** Some users would like to use this application offline, or they may have unreliable internet.
