mod media;

use std::{collections::{hash_map::Entry, HashMap}, error::Error, time::Duration};

use futures_lite::StreamExt;
use media::properties::{PlaybackStatus, Properties};
use tokio::time::{MissedTickBehavior, interval};
use zbus::{
    fdo::{DBusProxy, PropertiesProxy}, zvariant::{OwnedValue, Value}, Connection, Proxy
};

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
) -> Result<Properties, Box<dyn Error>> {
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
    Ok(properties)
}

async fn get_mpris_property(
    connection: &Connection,
    bus_name: &str,
    property_name: &str,
) -> Result<OwnedValue, Box<dyn Error>> {
    let reply = connection
        .call_method(
            Some(bus_name),
            "/org/mpris/MediaPlayer2",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.mpris.MediaPlayer2.Player", property_name),
        )
        .await?;

    let reply_body = reply.body();
    let value: Value = reply_body.deserialize()?;
    Ok(value.try_to_owned()?)
}

async fn get_mpris_position(
    connection: &Connection,
    bus_name: &str,
) -> Result<i64, Box<dyn Error>> {
    Ok(i64::try_from(
        get_mpris_property(connection, bus_name, "Position").await?,
    )?)
}

async fn get_mpris_playback_status(
    connection: &Connection,
    bus_name: &str,
) -> Result<PlaybackStatus, Box<dyn Error>> {
    Ok(PlaybackStatus::try_from(
        get_mpris_property(connection, bus_name, "PlaybackStatus").await?,
    )?)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;

    let proxy_properties = PropertiesProxy::new(
        &connection,
        "org.mpris.MediaPlayer2.spotify",
        "/org/mpris/MediaPlayer2",
    )
    .await?;
    let proxy_dbus = DBusProxy::new(&connection).await?;
    let mut properties_changed_signal = proxy_properties.receive_properties_changed().await?;
    let mut name_owner_changed = proxy_dbus.receive_name_owner_changed().await?;

    let mut interval = interval(Duration::from_secs(1));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    // // call interval.tick() here so that next tick will not execute immidiately
    // interval.tick().await;

    let bus_names = get_mpris_bus_names(&connection).await?;
    let mut bus_properties = HashMap::new();
    for bus_name in bus_names {
        bus_properties.insert(
            bus_name.clone(),
            get_mpris_all_properties(&connection, &bus_name).await?,
        );
    }
    // FIXME: firefox `Position` property incrementing while player is paused:
    // https://bugzilla.mozilla.org/show_bug.cgi?id=1950461
    //
    // https://phabricator.services.mozilla.com/D242633
    loop {
        if let Some(name) = name_owner_changed.next().await {
            let body = name.message().body();
            let message: (String, String, String) = body.deserialize()?;
            let bus_name = message.0;
            if bus_name.starts_with("org.mpris.MediaPlayer2") {
                match bus_properties.entry(bus_name.clone()) {
                    Entry::Occupied(occupied_entry) => {
                        occupied_entry.remove();
                    },
                    Entry::Vacant(vacant_entry) => {
                        let property = get_mpris_all_properties(&connection, &bus_name).await?;
                        vacant_entry.insert(property);
                    },
                }
                dbg!(&bus_properties);
            }
        }
        // interval.tick().await;
        // for bus_name in &bus_names {
        //     let property = properties.get_mut(bus_name).unwrap();
        //     property.position = get_mpris_position(&connection, bus_name).await?;
        //     property.playback_status = get_mpris_playback_status(&connection, bus_name).await?;
        //     dbg!(&bus_name, &property.position, &property.playback_status);
        // }
    }
}
