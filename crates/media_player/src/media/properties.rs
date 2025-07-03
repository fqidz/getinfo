use super::try_as_value::{self, try_as_optional};

use serde::Deserialize;
use zbus::zvariant::{
    OwnedValue, Type,
    as_value::{self, optional},
};

/// https://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata/
/// https://specifications.freedesktop.org/mpris-spec/latest/Track_List_Interface.html#Mapping:Metadata_Map
#[derive(Default, Deserialize, Type, Debug)]
#[zvariant(signature = "dict")]
#[serde(default)]
pub struct Metadata {
    #[serde(with = "try_as_value", rename(deserialize = "mpris:trackid"))]
    mpris_trackid: String,

    #[serde(
        with = "try_as_optional",
        rename(deserialize = "mpris:length"),
        skip_serializing_if = "Option::is_none"
    )]
    mpris_length: Option<i64>,

    #[serde(
        with = "optional",
        rename(deserialize = "mpris:artUrl"),
        skip_serializing_if = "Option::is_none"
    )]
    mpris_art_url: Option<String>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:album"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_album: Option<String>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:albumArtist"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_album_artist: Option<Vec<String>>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:artist"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_artist: Option<Vec<String>>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:asText"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_as_text: Option<String>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:audioBPM"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_audio_bpm: Option<i32>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:autoRating"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_auto_rating: Option<f64>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:comment"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_comment: Option<Vec<String>>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:composer"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_composer: Option<Vec<String>>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:contentCreated"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_content_created: Option<String>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:discNumber"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_disc_number: Option<i32>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:firstUsed"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_first_used: Option<String>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:genre"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_genre: Option<Vec<String>>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:lastUsed"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_last_used: Option<String>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:lyricist"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_lyricist: Option<Vec<String>>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:title"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_title: Option<String>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:trackNumber"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_track_number: Option<i32>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:url"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_url: Option<String>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:useCount"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_use_count: Option<i32>,

    #[serde(
        with = "optional",
        rename(deserialize = "xesam:userRating"),
        skip_serializing_if = "Option::is_none"
    )]
    xesam_user_rating: Option<f64>,
}

impl Metadata {
    /// A unique identity for this track within the context of an MPRIS object (eg: tracklist).
    ///
    /// Must always be present.
    pub fn trackid(&mut self) -> &mut String {
        &mut self.mpris_trackid
    }

    /// The duration of the track in microseconds.
    ///
    /// Present only if length is known.
    pub fn length(&mut self) -> Option<&mut i64> {
        self.mpris_length.as_mut()
    }

    /// The location of an image representing the track or album. Clients should not assume this
    /// will continue to exist when the media player stops giving out the URL.
    pub fn art_url(&mut self) -> Option<&mut String> {
        self.mpris_art_url.as_mut()
    }

    /// Album name
    pub fn album(&mut self) -> Option<&mut String> {
        self.xesam_album.as_mut()
    }

    /// The album artist(s).
    pub fn album_artist(&mut self) -> Option<&mut Vec<String>> {
        self.xesam_album_artist.as_mut()
    }

    /// The track artist(s).
    pub fn artist(&mut self) -> Option<&mut Vec<String>> {
        self.xesam_artist.as_mut()
    }

    /// The track lyrics.
    pub fn as_text(&mut self) -> Option<&mut String> {
        self.xesam_as_text.as_mut()
    }

    /// The speed of the music, in beats per minute.
    pub fn audio_bpm(&mut self) -> Option<&mut i32> {
        self.xesam_audio_bpm.as_mut()
    }

    /// An automatically-generated rating, based on things such as how often it has been played.
    /// This should be in the range 0.0 to 1.0.
    pub fn auto_rating(&mut self) -> Option<&mut f64> {
        self.xesam_auto_rating.as_mut()
    }

    /// A (list of) freeform comment(s).
    pub fn comment(&mut self) -> Option<&mut Vec<String>> {
        self.xesam_comment.as_mut()
    }

    /// The composer(s) of the track.
    pub fn composer(&mut self) -> Option<&mut Vec<String>> {
        self.xesam_composer.as_mut()
    }

    /// When the track was created. Usually only the year component will be useful.
    pub fn content_created(&mut self) -> Option<&mut String> {
        self.xesam_content_created.as_mut()
    }

    /// The disc number on the album that this track is from.
    pub fn disc_number(&self) -> Option<i32> {
        self.xesam_disc_number
    }

    /// When the track was first played.
    pub fn first_used(&mut self) -> Option<&mut String> {
        self.xesam_first_used.as_mut()
    }

    /// The genre(s) of the track.
    pub fn genre(&mut self) -> Option<&mut Vec<String>> {
        self.xesam_genre.as_mut()
    }

    /// When the track was last played.
    pub fn last_used(&mut self) -> Option<&mut String> {
        self.xesam_last_used.as_mut()
    }

    /// The lyricist(s) of the track.
    pub fn lyricist(&mut self) -> Option<&mut Vec<String>> {
        self.xesam_lyricist.as_mut()
    }

    /// The track title.
    pub fn title(&mut self) -> Option<&mut String> {
        self.xesam_title.as_mut()
    }

    /// The track number on the album disc.
    pub fn track_number(&mut self) -> Option<&mut i32> {
        self.xesam_track_number.as_mut()
    }

    /// The location of the media file.
    pub fn url(&mut self) -> Option<&mut String> {
        self.xesam_url.as_mut()
    }

    /// The number of times the track has been played.
    pub fn use_count(&mut self) -> Option<&mut i32> {
        self.xesam_use_count.as_mut()
    }

    /// A user-specified rating. This should be in the range 0.0 to 1.0.
    pub fn user_rating(&mut self) -> Option<&mut f64> {
        self.xesam_user_rating.as_mut()
    }
}

#[derive(Type, Deserialize, Debug, PartialEq, Eq)]
#[zvariant(signature = "s")]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

impl TryFrom<OwnedValue> for PlaybackStatus {
    type Error = zbus::zvariant::Error;

    fn try_from(value: OwnedValue) -> Result<Self, Self::Error> {
        match String::try_from(value)?.as_str() {
            "Playing" => Ok(Self::Playing),
            "Paused" => Ok(Self::Paused),
            "Stopped" => Ok(Self::Stopped),
            _ => Err(zbus::zvariant::Error::IncorrectType),
        }
    }
}

impl TryFrom<String> for PlaybackStatus {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Playing" => Ok(Self::Playing),
            "Paused" => Ok(Self::Paused),
            "Stopped" => Ok(Self::Stopped),
            _ => Err("Invalid playback status"),
        }
    }
}

#[derive(Type, Deserialize, Debug)]
#[zvariant(signature = "s")]
pub enum LoopStatus {
    None,
    Track,
    Playlist,
}

#[derive(Deserialize, Type, Debug)]
#[zvariant(signature = "dict")]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct Properties {
    #[serde(with = "as_value")]
    pub playback_status: PlaybackStatus,

    #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
    pub loop_status: Option<LoopStatus>,

    #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
    pub rate: Option<f64>,

    #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
    pub shuffle: Option<bool>,

    #[serde(with = "as_value")]
    pub metadata: Metadata,

    #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
    pub volume: Option<f64>,

    #[serde(with = "as_value")]
    pub position: i64,

    #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
    pub minimum_rate: Option<f64>,

    #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
    pub maximum_rate: Option<f64>,

    #[serde(with = "as_value")]
    pub can_go_next: bool,

    #[serde(with = "as_value")]
    pub can_go_previous: bool,

    #[serde(with = "as_value")]
    pub can_play: bool,

    #[serde(with = "as_value")]
    pub can_pause: bool,

    #[serde(with = "as_value")]
    pub can_seek: bool,

    #[serde(with = "as_value")]
    pub can_control: bool,
}
