mod media;

use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    future::pending,
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

use dashmap::{DashMap, Entry};
use futures_lite::StreamExt;
use media::properties::{PlaybackStatus, Properties};
use tokio::{
    sync::Mutex,
    time::{MissedTickBehavior, interval},
};
use zbus::{
    Connection, Proxy,
    fdo::{DBusProxy, PropertiesProxy},
    zvariant::{Dict, OwnedValue, Value},
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

    let proxy_dbus = DBusProxy::new(&connection).await?;
    let name_owner_changed = Arc::new(Mutex::new(proxy_dbus.receive_name_owner_changed().await?));

    let interval = Arc::new(Mutex::new(interval(Duration::from_secs(1))));
    interval
        .clone()
        .lock()
        .await
        .set_missed_tick_behavior(MissedTickBehavior::Skip);

    let bus_names = get_mpris_bus_names(&connection).await?;
    let bus_properties = Arc::new(DashMap::new());
    let property_streams = Arc::new(DashMap::new());

    for bus_name in bus_names {
        bus_properties.insert(
            bus_name.clone(),
            (
                get_mpris_all_properties(&connection, &bus_name).await?,
                None::<SystemTime>,
            ),
        );
        let proxy =
            PropertiesProxy::new(&connection, bus_name.clone(), "/org/mpris/MediaPlayer2").await?;
        let stream = proxy.receive_properties_changed().await?;
        property_streams.clone().insert(bus_name.clone(), stream);
    }

    // let properties_changed_signal = Arc::new(Mutex::new(
    //     proxy_properties.receive_properties_changed().await?,
    // ));

    // FIXME: firefox `Position` property incrementing while player is paused:
    // https://bugzilla.mozilla.org/show_bug.cgi?id=1950461
    //
    // https://phabricator.services.mozilla.com/D242633
    let name_owner_changed_handle = tokio::spawn({
        let name_owner_changed_cloned = name_owner_changed.clone();
        let property_streams_cloned = property_streams.clone();
        let bus_properties_cloned = bus_properties.clone();
        async move {
            loop {
                if let Some(name) = name_owner_changed_cloned.lock().await.next().await {
                    let body = name.message().body();
                    let message: (String, String, String) = body.deserialize().unwrap();
                    let bus_name = message.0;
                    if bus_name.starts_with("org.mpris.MediaPlayer2") {
                        match bus_properties_cloned.entry(bus_name.clone()) {
                            Entry::Occupied(occupied_entry) => {
                                property_streams_cloned.remove(&bus_name);
                                occupied_entry.remove();
                            }
                            Entry::Vacant(vacant_entry) => {
                                let property =
                                    get_mpris_all_properties(&connection.clone(), &bus_name)
                                        .await
                                        .unwrap();
                                let proxy = PropertiesProxy::new(
                                    &connection.clone(),
                                    bus_name.clone(),
                                    "/org/mpris/MediaPlayer2",
                                )
                                .await
                                .unwrap();
                                let stream = proxy.receive_properties_changed().await.unwrap();
                                property_streams_cloned.insert(bus_name.clone(), stream);
                                vacant_entry.insert((property, None));
                            }
                        }
                    }
                }
            }
        }
    });

    //
    // let playback_status_changed_handle = tokio::spawn({
    //     let proxy_properties_changed_signals_cloned = proxy_properties_changed_signals.clone();
    //     let bus_properties_cloned = bus_properties.clone();
    //     let interval_cloned = interval.clone();
    //     async move {
    //         loop {
    //             for mut pair in proxy_properties_changed_signals_cloned.iter_mut() {
    //                 let (bus_name, bus_proxy_properties_changed) = pair.pair_mut();
    //                 if let Some(properties_changed_message) =
    //                     bus_proxy_properties_changed.next().await
    //                 {
    //                     let body = properties_changed_message.message().body();
    //                     let mut message: (String, HashMap<String, Value>, Vec<String>) =
    //                         body.deserialize().unwrap();
    //                     dbg!(&message);
    //                     let playback_status = PlaybackStatus::try_from(
    //                         String::try_from(message.1.remove("PlaybackStatus").unwrap()).unwrap(),
    //                     )
    //                     .unwrap();
    //                     let mut bus_entry = bus_properties_cloned.get_mut(bus_name).unwrap();
    //                     let bus_property = bus_entry.value_mut();
    //                     bus_property.playback_status = playback_status;
    //                     dbg!(bus_property);
    //                     interval_cloned.lock().await.reset_immediately();
    //                 }
    //             }
    //         }
    //     }
    // });
    //
    // let update_position_handle = tokio::spawn({
    //     let bus_properties_cloned = bus_properties.clone();
    //     let conn_test = Connection::session().await.unwrap();
    //     let interval_cloned = interval.clone();
    //     async move {
    //         loop {
    //             interval_cloned.lock().await.tick().await;
    //             for mut pair in bus_properties_cloned.iter_mut() {
    //                 let (bus_name, bus_property) = pair.pair_mut();
    //                 if bus_property.playback_status == PlaybackStatus::Playing {
    //                     bus_property.position =
    //                         get_mpris_position(&connection, bus_name).await.unwrap();
    //                     dbg!(&bus_name, &bus_property);
    //                 }
    //             }
    //         }
    //     }
    // });

    pending::<()>().await;
    Ok(())
}
