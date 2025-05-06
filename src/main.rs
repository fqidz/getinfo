use std::{error::Error, future::pending};

use zbus::{connection, interface, Connection};

struct BarScripts {}

#[interface(name = "org.user.BarScripts")]
impl BarScripts {}

async fn get_mpris_bus_names(connection: &Connection) -> Result<Vec<String>, Box<dyn Error>> {
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
    Ok(all_bus_names
        .into_iter()
        .filter(|bus_name| bus_name.starts_with("org.mpris.MediaPlayer2"))
        .collect::<Vec<_>>())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let bar_scripts = BarScripts {};
    let connection = connection::Builder::session()?
        .name("org.user.BarScripts")?
        .serve_at("/org/user/BarScripts", bar_scripts)?
        .build()
        .await?;

    eprintln!("{:?}", get_mpris_bus_names(&connection).await?);

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
