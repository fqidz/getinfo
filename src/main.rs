use std::{error::Error, future::pending};

use zbus::{connection, interface};

struct BarScripts {}

#[interface(name = "org.user.BarScripts")]
impl BarScripts {}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let bar_scripts = BarScripts {};
    let connection = connection::Builder::session()?
        .name("org.user.BarScripts")?
        .serve_at("/org/user/BarScripts", bar_scripts)?
        .build()
        .await?;
    let reply = connection
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "ListNames",
            &(),
        )
        .await?;
    let all_bus_names = reply.body().deserialize::<Vec<String>>()?;
    dbg!(
        all_bus_names
            .iter()
            .filter(|bus_name| bus_name.starts_with("org.mpris.MediaPlayer2"))
            .collect::<Vec<_>>()
    );

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
