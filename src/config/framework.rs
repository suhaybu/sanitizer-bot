use super::{Data, Error};
use crate::commands::age::age;
use poise::{Framework, FrameworkOptions};

// pub fn get_options() -> Result<FrameworkOptions<Data, Error>, Error> {
//     let options = FrameworkOptions {
//         commands: vec![age()],
//         ..Default::default()
//     };

//     Ok(options)
// }

pub fn create_framework() -> Result<Framework<Data, Error>, Error> {
    let my_commands = vec![age()];

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: my_commands,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    Ok(framework)
}
