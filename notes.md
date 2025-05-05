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

