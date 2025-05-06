# notes
## Sources
- [DBus-Specification#standard-interfaces](https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces)
- [MPRIS DBus Interface Specification](https://specifications.freedesktop.org/mpris-spec/latest/)
- [Example of using dbus-send to get properties](https://stackoverflow.com/questions/36162845/how-to-get-properties-through-gdbus)

## Getting Properties
```
org.freedesktop.DBus.Properties.Get (in STRING interface_name,
                                   in STRING property_name,
                                   out VARIANT value);
org.freedesktop.DBus.Properties.Set (in STRING interface_name,
                                   in STRING property_name,
                                   in VARIANT value);
org.freedesktop.DBus.Properties.GetAll (in STRING interface_name,
                                      out ARRAY of DICT_ENTRY<STRING,VARIANT> props);
```

### Get track position from spotify
- [MPRIS Bus Name Policy](https://specifications.freedesktop.org/mpris-spec/latest/#Bus-Name-Policy)

```bash
dbus-send \
    --print-reply \
    --dest=org.mpris.MediaPlayer2.spotify    `# Bus name`       \
    /org/mpris/MediaPlayer2                  `# Object path`    \
    org.freedesktop.DBus.Properties.Get      `# Method to call` \
    string:'org.mpris.MediaPlayer2.Player'   `# arg0`           \
    string:'Position'                        `# arg1`           \
```

### Keeping track of MPRIS bus names
#### On script startup
When we start the script, we first need to list all the bus names open, so that
we can find `org.mpris.MediaPlayer2.*` bus names:
```
$ dbus-send --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.ListNames
method return time=1746440421.330679 sender=org.freedesktop.DBus -> destination=:1.213 serial=4294967295 reply_serial=2
   array [
      string "org.freedesktop.DBus"
      string ":1.2"
      ...
      string ":1.213"
      string "ca.desrt.dconf"
      string "org.freedesktop.Notifications"
      string "org.freedesktop.ReserveDevice1.Audio0"
      string "org.freedesktop.ReserveDevice1.Audio1"
      string "org.freedesktop.ReserveDevice1.Audio2"
      string "org.freedesktop.ScreenSaver"
      string "org.freedesktop.impl.portal.PermissionStore"
      string "org.freedesktop.impl.portal.desktop.hyprland"
      string "org.freedesktop.portal.Desktop"
      string "org.freedesktop.portal.Documents"
      string "org.freedesktop.systemd1"
      string "org.mozilla.firefox.L2hv...Nzk_"
      string "org.mpris.MediaPlayer2.firefox.instance_1_9"
      string "org.mpris.MediaPlayer2.spotify"
      string "org.pipewire.Telephony"
      string "org.pulseaudio.Server"
   ]
```

Here we can see that `org.mpris.MediaPlayer2.spotify` and
`org.mpris.MediaPlayer2.firefox.instance_1_9` (a YouTube video on Firefox) is
open.

#### While script is running
Once we have the initial bus names, we need to keep track of new ones that
open. Because, for example, we need to detect if we open a youtube video in
Firefox while the script is already running. We can detect this by listening to
the `org.freedesktop.DBus.NameAcquired` signal [(defined here)](https://dbus.freedesktop.org/doc/dbus-specification.html#message-bus-messages)

Running `dbus-monitor` and then opening Spotify, we can see that the
`NameAcquired` signal -- which includes the bus name in its value -- is emitted
after the `RequestName` method is called:

```
...
method call time=1746439584.456129 sender=:1.175 -> destination=org.freedesktop.DBus serial=30 path=/org/freedesktop/DBus;
interface=org.freedesktop.DBus;
member=RequestName
   string "org.mpris.MediaPlayer2.spotify"
   uint32 0

signal time=1746439584.456134 sender=org.freedesktop.DBus -> destination=(null destination) serial=4294967295 path=/org/freedesktop/DBus;
interface=org.freedesktop.DBus;
member=NameOwnerChanged
   string "org.mpris.MediaPlayer2.spotify"
   string ""
   string ":1.175"

signal time=1746439584.456141 sender=org.freedesktop.DBus -> destination=:1.175 serial=4294967295 path=/org/freedesktop/DBus;
interface=org.freedesktop.DBus;
member=NameAcquired
   string "org.mpris.MediaPlayer2.spotify"
...
```

I'm not sure if there's a way to directly read the `RequestName` method call,
so we read the `NameAcquired` signal instead
