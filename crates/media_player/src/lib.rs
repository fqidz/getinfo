use std::{
    collections::HashSet,
    sync::mpsc::{self, Sender}, thread,
};

use zbus::{
    blocking::Connection,
    zvariant::{OwnedValue, Value},
};

use crate::media::properties::{LoopStatus, Metadata, PlaybackStatus};

mod media;
//
// use std::{
//     cell::RefCell,
//     collections::HashMap,
//     error::Error,
//     future::pending,
//     sync::{mpsc::Sender, Arc},
//     time::{Duration, Instant, SystemTime},
// };
//
// use dashmap::{DashMap, Entry};
// use futures_lite::StreamExt;
// use media::properties::{PlaybackStatus, Properties};
// use tokio::{
//     sync::Mutex,
//     time::{MissedTickBehavior, interval},
// };
// use zbus::{
//     Connection, Proxy,
//     fdo::{DBusProxy, PropertiesProxy},
//     zvariant::{Dict, OwnedValue, Value},
// };

pub struct MediaPlayer {
    connection: zbus::blocking::Connection,
    watched_properties: HashSet<PropertyName>,
    sender: Sender<Property>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum MetadataName {
    MprisTrackid,
    MprisLength,
    MprisArtUrl,
    XesamAlbum,
    XesamAlbumArtist,
    XesamArtist,
    XesamAsText,
    XesamAudioBpm,
    XesamAutoRating,
    XesamComment,
    XesamComposer,
    XesamContentCreated,
    XesamDiscNumber,
    XesamFirstUsed,
    XesamGenre,
    XesamLastUsed,
    XesamLyricist,
    XesamTitle,
    XesamTrackNumber,
    XesamUrl,
    XesamUseCount,
    XesamUserRating,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum PropertyName {
    PlaybackStatus,
    LoopStatus,
    Rate,
    Shuffle,
    Metadata(MetadataName),
    Volume,
    Position,
    MinimumRate,
    MaximumRate,
    CanGoNext,
    CanGoPrevious,
    CanPlay,
    CanPause,
    CanSeek,
    CanControl,
}

#[derive(Debug)]
pub struct Property {
    name: PropertyName,
    value: OwnedValue,
}

#[derive(Default)]
pub struct MediaPlayerBuilder {
    watched_properties: HashSet<PropertyName>,
}

impl MediaPlayerBuilder {
    pub fn build_and_start(self, sender: Sender<Property>) -> MediaPlayer {
        assert!(!self.watched_properties.is_empty());

        let connection = Connection::session().unwrap();
        let media_player = MediaPlayer {
            connection,
            sender,
            watched_properties: self.watched_properties,
        };
        media_player.start();
        media_player
    }

    pub fn watch(mut self, property: PropertyName) -> Self {
        self.watched_properties.insert(property);
        self
    }

    pub fn watch_multiple(mut self, properties: Vec<PropertyName>) -> Self {
        self.watched_properties.extend(properties);
        self
    }
}

impl MediaPlayer {
    pub fn builder() -> MediaPlayerBuilder {
        MediaPlayerBuilder::default()
    }

    pub(crate) fn start(&self) {
        let tx = self.sender.clone();
        let _ = thread::Builder::new()
            .name("media player watch loop".to_string())
            .spawn(move || {
                loop {
                    tx.send(Property { name: PropertyName::LoopStatus, value: OwnedValue::from(123) }).unwrap();
                }
            });
    }
}

pub fn foo() {
    let (tx, rx) = mpsc::channel::<Property>();
    let _media_player = MediaPlayerBuilder::default()
        .watch(PropertyName::LoopStatus)
        .watch(PropertyName::Volume)
        .build_and_start(tx);

    for prop in rx {
        dbg!(prop);
    }
}

// match property {
//     PropertyName::PlaybackStatus => todo!(),
//     PropertyName::LoopStatus => todo!(),
//     PropertyName::Rate => todo!(),
//     PropertyName::Shuffle => todo!(),
//     PropertyName::Metadata(metadata_name) => {
//         match metadata_name {
//             MetadataName::MprisTrackid => todo!(),
//             MetadataName::MprisLength => todo!(),
//             MetadataName::MprisArtUrl => todo!(),
//             MetadataName::XesamAlbum => todo!(),
//             MetadataName::XesamAlbumArtist => todo!(),
//             MetadataName::XesamArtist => todo!(),
//             MetadataName::XesamAsText => todo!(),
//             MetadataName::XesamAudioBpm => todo!(),
//             MetadataName::XesamAutoRating => todo!(),
//             MetadataName::XesamComment => todo!(),
//             MetadataName::XesamComposer => todo!(),
//             MetadataName::XesamContentCreated => todo!(),
//             MetadataName::XesamDiscNumber => todo!(),
//             MetadataName::XesamFirstUsed => todo!(),
//             MetadataName::XesamGenre => todo!(),
//             MetadataName::XesamLastUsed => todo!(),
//             MetadataName::XesamLyricist => todo!(),
//             MetadataName::XesamTitle => todo!(),
//             MetadataName::XesamTrackNumber => todo!(),
//             MetadataName::XesamUrl => todo!(),
//             MetadataName::XesamUseCount => todo!(),
//             MetadataName::XesamUserRating => todo!(),
//         }
//     },
//     PropertyName::Volume => todo!(),
//     PropertyName::Position => todo!(),
//     PropertyName::MinimumRate => todo!(),
//     PropertyName::MaximumRate => todo!(),
//     PropertyName::CanGoNext => todo!(),
//     PropertyName::CanGoPrevious => todo!(),
//     PropertyName::CanPlay => todo!(),
//     PropertyName::CanPause => todo!(),
//     PropertyName::CanSeek => todo!(),
//     PropertyName::CanControl => todo!(),
// }

// #[derive(Deserialize, Type, Debug)]
// #[zvariant(signature = "dict")]
// #[serde(rename_all(deserialize = "PascalCase"))]
// pub struct Properties {
//     #[serde(with = "as_value")]
//     pub playback_status: PlaybackStatus,
//
//     #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
//     pub loop_status: Option<LoopStatus>,
//
//     #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
//     pub rate: Option<f64>,
//
//     #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
//     pub shuffle: Option<bool>,
//
//     #[serde(with = "as_value")]
//     pub metadata: Metadata,
//
//     #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
//     pub volume: Option<f64>,
//
//     #[serde(with = "as_value")]
//     pub position: i64,
//
//     #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
//     pub minimum_rate: Option<f64>,
//
//     #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
//     pub maximum_rate: Option<f64>,
//
//     #[serde(with = "as_value")]
//     pub can_go_next: bool,
//
//     #[serde(with = "as_value")]
//     pub can_go_previous: bool,
//
//     #[serde(with = "as_value")]
//     pub can_play: bool,
//
//     #[serde(with = "as_value")]
//     pub can_pause: bool,
//
//     #[serde(with = "as_value")]
//     pub can_seek: bool,
//
//     #[serde(with = "as_value")]
//     pub can_control: bool,
// }

// pub async fn get_mpris_bus_names(connection: &Connection) -> Result<Vec<String>, Box<dyn Error>> {
//     let reply = connection
//         .call_method(
//             Some("org.freedesktop.DBus"),
//             "/org/freedesktop/DBus",
//             Some("org.freedesktop.DBus"),
//             "ListNames",
//             &(),
//         )
//         .await?;
//     let all_bus_names: Vec<String> = reply.body().deserialize()?;
//     Ok(all_bus_names
//         .into_iter()
//         .filter(|bus_name| bus_name.starts_with("org.mpris.MediaPlayer2"))
//         .collect::<Vec<_>>())
// }
//
// pub async fn get_mpris_all_properties(
//     connection: &Connection,
//     bus_name: &str,
// ) -> Result<Properties, Box<dyn Error>> {
//     let reply = connection
//         .call_method(
//             Some(bus_name),
//             "/org/mpris/MediaPlayer2",
//             Some("org.freedesktop.DBus.Properties"),
//             "GetAll",
//             &("org.mpris.MediaPlayer2.Player"),
//         )
//         .await?;
//
//     let reply_body = reply.body();
//     let properties: Properties = reply_body.deserialize()?;
//     Ok(properties)
// }
//
// pub async fn get_mpris_property(
//     connection: &Connection,
//     bus_name: &str,
//     property_name: &str,
// ) -> Result<OwnedValue, Box<dyn Error>> {
//     let reply = connection
//         .call_method(
//             Some(bus_name),
//             "/org/mpris/MediaPlayer2",
//             Some("org.freedesktop.DBus.Properties"),
//             "Get",
//             &("org.mpris.MediaPlayer2.Player", property_name),
//         )
//         .await?;
//
//     let reply_body = reply.body();
//     let value: Value = reply_body.deserialize()?;
//     Ok(value.try_to_owned()?)
// }
//
// pub async fn get_mpris_position(
//     connection: &Connection,
//     bus_name: &str,
// ) -> Result<i64, Box<dyn Error>> {
//     Ok(i64::try_from(
//         get_mpris_property(connection, bus_name, "Position").await?,
//     )?)
// }
//
// pub async fn get_mpris_playback_status(
//     connection: &Connection,
//     bus_name: &str,
// ) -> Result<PlaybackStatus, Box<dyn Error>> {
//     Ok(PlaybackStatus::try_from(
//         get_mpris_property(connection, bus_name, "PlaybackStatus").await?,
//     )?)
// }
//
// async fn main() -> Result<(), Box<dyn Error>> {
//     let connection = Connection::session().await?;
//
//     let proxy_dbus = DBusProxy::new(&connection).await?;
//     let name_owner_changed = Arc::new(Mutex::new(proxy_dbus.receive_name_owner_changed().await?));
//
//     let interval = Arc::new(Mutex::new(interval(Duration::from_secs(1))));
//     interval
//         .clone()
//         .lock()
//         .await
//         .set_missed_tick_behavior(MissedTickBehavior::Skip);
//
//     let bus_names = get_mpris_bus_names(&connection).await?;
//     let bus_properties = Arc::new(DashMap::new());
//     let property_streams = Arc::new(DashMap::new());
//
//     for bus_name in bus_names {
//         bus_properties.insert(
//             bus_name.clone(),
//             (
//                 get_mpris_all_properties(&connection, &bus_name).await?,
//                 None::<SystemTime>,
//             ),
//         );
//         let proxy =
//             PropertiesProxy::new(&connection, bus_name.clone(), "/org/mpris/MediaPlayer2").await?;
//         let stream = proxy.receive_properties_changed().await?;
//         property_streams.clone().insert(bus_name.clone(), stream);
//     }
//
//     // let properties_changed_signal = Arc::new(Mutex::new(
//     //     proxy_properties.receive_properties_changed().await?,
//     // ));
//
//     // FIXME: firefox `Position` property incrementing while player is paused:
//     // https://bugzilla.mozilla.org/show_bug.cgi?id=1950461
//     //
//     // https://phabricator.services.mozilla.com/D242633
//     let name_owner_changed_handle = tokio::spawn({
//         let name_owner_changed_cloned = name_owner_changed.clone();
//         let property_streams_cloned = property_streams.clone();
//         let bus_properties_cloned = bus_properties.clone();
//         async move {
//             loop {
//                 if let Some(name) = name_owner_changed_cloned.lock().await.next().await {
//                     let body = name.message().body();
//                     let message: (String, String, String) = body.deserialize().unwrap();
//                     let bus_name = message.0;
//                     if bus_name.starts_with("org.mpris.MediaPlayer2") {
//                         match bus_properties_cloned.entry(bus_name.clone()) {
//                             Entry::Occupied(occupied_entry) => {
//                                 property_streams_cloned.remove(&bus_name);
//                                 occupied_entry.remove();
//                             }
//                             Entry::Vacant(vacant_entry) => {
//                                 let property =
//                                     get_mpris_all_properties(&connection.clone(), &bus_name)
//                                         .await
//                                         .unwrap();
//                                 let proxy = PropertiesProxy::new(
//                                     &connection.clone(),
//                                     bus_name.clone(),
//                                     "/org/mpris/MediaPlayer2",
//                                 )
//                                 .await
//                                 .unwrap();
//                                 let stream = proxy.receive_properties_changed().await.unwrap();
//                                 property_streams_cloned.insert(bus_name.clone(), stream);
//                                 vacant_entry.insert((property, None));
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     });
//
//     //
//     // let playback_status_changed_handle = tokio::spawn({
//     //     let proxy_properties_changed_signals_cloned = proxy_properties_changed_signals.clone();
//     //     let bus_properties_cloned = bus_properties.clone();
//     //     let interval_cloned = interval.clone();
//     //     async move {
//     //         loop {
//     //             for mut pair in proxy_properties_changed_signals_cloned.iter_mut() {
//     //                 let (bus_name, bus_proxy_properties_changed) = pair.pair_mut();
//     //                 if let Some(properties_changed_message) =
//     //                     bus_proxy_properties_changed.next().await
//     //                 {
//     //                     let body = properties_changed_message.message().body();
//     //                     let mut message: (String, HashMap<String, Value>, Vec<String>) =
//     //                         body.deserialize().unwrap();
//     //                     dbg!(&message);
//     //                     let playback_status = PlaybackStatus::try_from(
//     //                         String::try_from(message.1.remove("PlaybackStatus").unwrap()).unwrap(),
//     //                     )
//     //                     .unwrap();
//     //                     let mut bus_entry = bus_properties_cloned.get_mut(bus_name).unwrap();
//     //                     let bus_property = bus_entry.value_mut();
//     //                     bus_property.playback_status = playback_status;
//     //                     dbg!(bus_property);
//     //                     interval_cloned.lock().await.reset_immediately();
//     //                 }
//     //             }
//     //         }
//     //     }
//     // });
//     //
//     // let update_position_handle = tokio::spawn({
//     //     let bus_properties_cloned = bus_properties.clone();
//     //     let conn_test = Connection::session().await.unwrap();
//     //     let interval_cloned = interval.clone();
//     //     async move {
//     //         loop {
//     //             interval_cloned.lock().await.tick().await;
//     //             for mut pair in bus_properties_cloned.iter_mut() {
//     //                 let (bus_name, bus_property) = pair.pair_mut();
//     //                 if bus_property.playback_status == PlaybackStatus::Playing {
//     //                     bus_property.position =
//     //                         get_mpris_position(&connection, bus_name).await.unwrap();
//     //                     dbg!(&bus_name, &bus_property);
//     //                 }
//     //             }
//     //         }
//     //     }
//     // });
//
//     pending::<()>().await;
//     Ok(())
// }
