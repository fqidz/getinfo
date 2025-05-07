mod media;

use std::{
    error::Error, time::Duration,
};

use media::properties::Properties;
use tokio::time::interval;
use zbus::{
    Connection, connection as conn, interface,
};

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
    let all_bus_names: Vec<String> = reply.body().deserialize()?;
    Ok(all_bus_names
        .into_iter()
        .filter(|bus_name| bus_name.starts_with("org.mpris.MediaPlayer2"))
        .collect::<Vec<_>>())
}

async fn get_mpris_all_properties(
    connection: &Connection,
    bus_name: &str,
) -> Result<(), Box<dyn Error>> {
    let reply = connection
        .call_method(
            Some(bus_name),
            "/org/mpris/MediaPlayer2",
            Some("org.freedesktop.DBus.Properties"),
            "GetAll",
            &("org.mpris.MediaPlayer2.Player"),
        )
        .await?;

    let reply_body = reply.body();
    let properties: Properties = reply_body.deserialize()?;
    dbg!(properties);
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let bar_scripts = BarScripts {};
    let connection = conn::Builder::session()?
        .name("org.user.BarScripts")?
        .serve_at("/org/user/BarScripts", bar_scripts)?
        .build()
        .await?;

    let mut interval = interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let bus_names = get_mpris_bus_names(&connection).await?;
        println!("{:?}", bus_names);
        for bus_name in bus_names {
            get_mpris_all_properties(&connection, &bus_name).await?;
        }
    }
}
